use serde::{Deserialize, Serialize};

/// Type used for database requests.
#[derive(Deserialize, Serialize, Debug)]
pub enum DbRequest {
    /// Get a single value as an `Option<Vec<u8>>`.
    Get {
        owner: String,
        table: String,
        key: String,
    },
    /// Set a single value.
    Set {
        owner: String,
        table: String,
        key: String,
        value: Vec<u8>,
    },
    /// Delete a single value.
    Del {
        owner: String,
        table: String,
        key: String,
    },
    /// Get all values with a prefix.
    GetPrefixed {
        owner: String,
        table: String,
        prefix: String,
    },
    /// Delete all values with a prefix.
    DelPrefixed {
        owner: String,
        table: String,
        prefix: String,
    },
    /// Set multiple key-value pairs.
    SetMany {
        owner: String,
        table: String,
        pairs: Vec<(String, Vec<u8>)>,
    },
}
/// Type used for database responses.
#[derive(Deserialize, Serialize, Debug)]
pub enum DbResponse {
    Value(Vec<u8>),
    SetOk,
    DeletedOk,
    ManyValues(Vec<Vec<u8>>),
    DeletedManyOk(usize),
    SetManyOk,
    InternalError(String),
}
