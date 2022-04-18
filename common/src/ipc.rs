use log::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::errors::IpcError;

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
        trace!("Awaiting readable stream...");
        self.stream.readable().await?;
        trace!("Stream readable.");
        let len = match self.stream.read(&mut self.buf).await {
            Ok(0) => {
                warn!("Read 0, client disconnected.");
                return Err(IpcError::ClientDisconnected);
            }
            Ok(l) => {
                trace!("Read request of length {}B", l);
                l
            }
            Err(e) => {
                error!("IO error while reading request: {}", e);
                return Err(e.into());
            }
        };
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
        match self.stream.write(&request_bytes).await {
            Ok(0) => {
                error!("Server disconnected while writing response.");
                return Err(IpcError::ServerDisconnected);
            }
            Ok(_) => (),
            Err(e) => {
                error!("IO error while writing request: {}", e);
                return Err(e.into());
            }
        };
        trace!("Awaiting readable stream...");
        self.stream.readable().await?;
        trace!("Stream readable.");
        let len = match self.stream.read(&mut self.buf).await {
            Ok(0) => {
                error!("Server disconnected while reading response.");
                return Err(IpcError::ServerDisconnected);
            }
            Ok(l) => {
                trace!("Received response of length {}B", l);
                l
            }
            Err(e) => {
                error!("IO error while reading response: {}", e);
                return Err(e.into());
            }
        };
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
