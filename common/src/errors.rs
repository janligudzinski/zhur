use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("The client disconnected.")]
    ClientDisconnected,
    #[error("Data could not be read from or written to the client.")]
    ClientConnection(#[from] std::io::Error),
    #[error("The request could not be deserialized.")]
    RequestDeserialization,
    #[error("The response could not be serialized.")]
    ResponseSerialization,
}
