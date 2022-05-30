use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};

use crate::__internals::wapc_guest as wapc;
use crate::error::DbError;
/// Get a single value from a given table by its key.
pub fn get<T: DeserializeOwned>(table: &str, key: &str) -> Result<Option<T>, DbError> {
    let request = serialize(&(table, key)).unwrap();
    let response = wapc::host_call("", "db", "get", &request).unwrap();
    let response =
        deserialize::<Option<Vec<u8>>>(&response).map_err(|_| DbError::DeserializationError)?;
    match response {
        None => Ok(None),
        Some(bytes) => deserialize::<Option<T>>(&bytes).map_err(|_| DbError::DeserializationError),
    }
}
/// Set a single value at the given table and key.
pub fn set<T: Serialize>(table: &str, key: &str, value: &T) -> Result<(), DbError> {
    unimplemented!()
}
/// Delete a single value.
pub fn del(table: &str, key: &str) -> Result<(), DbError> {
    unimplemented!()
}
/// Retrieve all values in a table whose keys start with the given prefix.
pub fn get_prefixed<T: DeserializeOwned>(table: &str, key_prefix: &str) -> Result<Vec<T>, DbError> {
    unimplemented!()
}
/// Delete all values in a table whose keys start with the given prefix. Returns how many keys were deleted.
pub fn del_prefixed(table: &str, key_prefix: &str) -> Result<usize, DbError> {
    unimplemented!()
}
/// Set multiple key-value pairs.
pub fn set_many<T: Serialize>(table: &str, pairs: &(&str, &T)) -> Result<(), DbError> {
    unimplemented!()
}
