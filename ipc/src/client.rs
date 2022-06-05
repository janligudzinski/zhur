use log::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::UnixStream,
};

use common::errors::IpcError;

pub struct UnixClient {
    /// Internal message buffer.
    buf: Vec<u8>,
    /// Underlying Unix socket stream.
    stream: UnixStream,
}

impl UnixClient {
    pub fn new(buffer_size: usize, stream: UnixStream) -> Self {
        Self {
            buf: vec![0; buffer_size],
            stream,
        }
    }
    pub async fn request<Req: Serialize, Res: DeserializeOwned>(
        &mut self,
        request: &Req,
    ) -> Result<Res, IpcError> {
        trace!("Clearing response read buffer.");
        self.buf.fill(0); // clear() sets the length to 0. This results in false positives for disconnection detection.
        trace!("Awaiting writable stream...");
        self.stream.writable().await?;
        trace!("Stream writable.");
        let request_bytes = match bincode::serialize(request) {
            Ok(v) => v,
            Err(_) => {
                error!("Could not serialize request");
                return Err(IpcError::RequestSerialization);
            }
        };
        self.stream
            .write(&request_bytes.len().to_be_bytes())
            .await?;
        info!("Notified other end of {}B request", &request_bytes.len());
        match self.stream.write_all(&request_bytes).await {
            Ok(()) => {
                info!(
                    "Wrote request of size {}B successfully.",
                    request_bytes.len()
                );
            }
            Err(e) => {
                error!("IO error while writing request: {}", e);
                return Err(e.into());
            }
        };
        let intended_len = self.stream.read_u64().await? as usize;
        info!("Response length is expected to be {}B", intended_len);
        let mut len = 0usize;
        loop {
            trace!("Awaiting readable stream to be ready...");
            let ready = self.stream.ready(Interest::READABLE).await?;
            if !ready.is_readable() {
                continue;
            }
            trace!("Stream readable.");
            match self.stream.try_read(&mut self.buf) {
                Ok(0) => {
                    error!("Server disconnected while client was reading response.");
                    return Err(IpcError::ServerDisconnected);
                }
                Ok(l) => {
                    trace!(
                        "Received response chunk of length {}B - {}B / {}B",
                        l,
                        len,
                        intended_len
                    );
                    len += l;
                }
                Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                    if len < intended_len {
                        continue;
                    } else {
                        trace!("Reading response would block, assuming finished. Total response length {}B", len);
                        break;
                    }
                }
                Err(e) => {
                    error!("IO error while reading response: {}", e);
                    return Err(e.into());
                }
            }
        }
        let result = match bincode::deserialize::<Res>(&self.buf[0..len]) {
            Ok(r) => Ok(r),
            Err(_) => {
                error!("Could not deserialize response.");
                Err(IpcError::ResponseDeserialization)
            }
        };
        result
    }
}
