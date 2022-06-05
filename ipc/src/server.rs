use log::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::UnixStream,
};

use common::errors::IpcError;

pub struct UnixServer {
    /// Internal message buffer.
    buf: Vec<u8>,
    /// Underlying Unix socket stream.
    stream: UnixStream,
}

impl UnixServer {
    pub fn new(buffer_size: usize, stream: UnixStream) -> Self {
        Self {
            buf: vec![0; buffer_size],
            stream,
        }
    }
    pub async fn get_request<Req: DeserializeOwned>(&mut self) -> Result<Req, IpcError> {
        trace!("Clearing request read buffer.");
        self.buf.fill(0); // clear() sets the length to 0. This results in false positives for disconnection detection.
        let intended_len = self.stream.read_u64().await? as usize;
        info!("Intended request length is {}B", intended_len);
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
                    warn!("Read 0, client disconnected.");
                    return Err(IpcError::ClientDisconnected);
                }
                Ok(l) => {
                    trace!(
                        "Read request chunk of length {}B - {}B / {}B",
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
                    error!("IO error while reading request: {}", e);
                    return Err(e.into());
                }
            }
        }
        let result = match bincode::deserialize::<Req>(&self.buf[0..len]) {
            Ok(r) => Ok(r),
            Err(_) => {
                error!("Could not deserialize request type.");
                Err(IpcError::RequestDeserialization)
            }
        };
        result
    }
    pub async fn send_response<Res: Serialize>(&mut self, response: &Res) -> Result<(), IpcError> {
        trace!("Awaiting writable stream...");
        self.stream.writable().await?;
        trace!("Stream writable.");
        let response_bytes = match bincode::serialize(response) {
            Ok(v) => v,
            Err(_) => {
                error!("Could not serialize response.");
                return Err(IpcError::ResponseSerialization);
            }
        };
        self.stream
            .write(&response_bytes.len().to_be_bytes())
            .await?;
        info!("Notified other end of {}B response.", response_bytes.len());
        match self.stream.write_all(&response_bytes).await {
            Ok(_) => {
                trace!("Wrote a response.");
                Ok(())
            }
            Err(e) => {
                error!("IO error while writing response: {}", e);
                Err(e.into())
            }
        }
    }
}
