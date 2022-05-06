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

#[derive(Debug, Error)]
pub enum InvocationError {
    #[error("The HTTP request being used to create an invocation was malformed.")]
    BadHttpRequest,
    #[error("The application returned an internal error: {0}")]
    ApplicationError(String),
    #[error(
        "No application named {0}:{1} was found. It may not exist or be temporarily disabled."
    )]
    NoAppFound(String, String),
}
