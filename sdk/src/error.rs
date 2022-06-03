use thiserror::Error;

/// Errors that can arise when dealing with the key-value database.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("The value retrieved could not be deserialized as the given type.")]
    DeserializationError,
    #[error("The database encountered an internal error: {0}")]
    Internal(String),
}
