use serde::{Deserialize, Serialize};
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

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum InvocationError {
    #[error("The HTTP request being used to create an invocation was malformed.")]
    BadHttpRequest,
    #[error("The application returned an internal error: {0}")]
    ApplicationError(String),
    #[error(
        "No application named {0}:{1} was found. It may not exist or be temporarily disabled."
    )]
    NoAppFound(String, String),
    #[error("The invocation and response types did not match - the application returned an HTTP response to a plaintext invocation or vice versa.")]
    InvokeTypeMismatch,
    #[error("The payload for a text function was not valid serializable UTF-8 text.")]
    InvalidTextPayload,
    #[error("The return value of a text function could not be deserialized as valid UTF-8 text.")]
    InvalidTextOutput,
    #[error("Internal WASM code execution error: {0}")]
    ExecutionError(String),
    #[error("A WASM host could not be spawned. Details: {0}")]
    HostInitializationError(String),
    #[error("The WASM code provided could not be loaded. Details: {0}")]
    BadCode(String),
}
