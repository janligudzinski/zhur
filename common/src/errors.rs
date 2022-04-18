use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("The client disconnected.")]
    ClientDisconnected,
    #[error("Data could not be read from or written to the client or server.")]
    Connection(#[from] std::io::Error),
    #[error("The request could not be deserialized.")]
    RequestDeserialization,
    #[error("The request could not be serialized.")]
    RequestSerialization,
    #[error("The response could not be deserialized.")]
    ResponseDeserialization,
    #[error("The response could not be serialized.")]
    ResponseSerialization,
    #[error("The server disconnected.")]
    ServerDisconnected,
}
