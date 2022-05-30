use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use wapc::host_call;

use crate::__internals::wapc_guest as wapc;
use crate::error::DbError;
/// Get a single value from a given table by its key.
pub fn get<T: DeserializeOwned>(table: &str, key: &str) -> Result<Option<T>, DbError> {
    let request = serialize(&(table, key)).unwrap();
    let response = host_call("", "db", "get", &request).unwrap();
    let response =
        deserialize::<Option<Vec<u8>>>(&response).map_err(|_| DbError::DeserializationError)?;
    match response {
        None => Ok(None),
        Some(bytes) => deserialize::<Option<T>>(&bytes).map_err(|_| DbError::DeserializationError),
    }
}
/// Set a single value at the given table and key.
pub fn set<T: Serialize>(table: &str, key: &str, value: &T) -> Result<(), DbError> {
    let bytes = serialize(value).unwrap();
    let request = serialize(&(table, key, bytes)).unwrap();
    match host_call("", "db", "set", &request) {
        Ok(_) => Ok(()),
        Err(e) => Err(DbError::Internal(e.to_string())),
    }
}
/// Delete a single value.
pub fn del(table: &str, key: &str) -> Result<(), DbError> {
    let request = serialize(&(table, key)).unwrap();
    match host_call("", "db", "del", &request) {
        Ok(_) => Ok(()),
        Err(e) => Err(DbError::Internal(e.to_string())),
    }
}
/// Retrieve all values in a table whose keys start with the given prefix.
pub fn get_prefixed<T: DeserializeOwned>(table: &str, key_prefix: &str) -> Result<Vec<T>, DbError> {
    let request = serialize(&(table, key_prefix)).unwrap();
    match host_call("", "db", "get_prefixed", &request) {
        Ok(bytes) => deserialize::<Vec<T>>(&bytes).map_err(|_| DbError::DeserializationError),
        Err(e) => Err(DbError::Internal(e.to_string())),
    }
}
/// Delete all values in a table whose keys start with the given prefix. Returns how many keys were deleted.
pub fn del_prefixed(table: &str, key_prefix: &str) -> Result<u64, DbError> {
    let request = serialize(&(table, key_prefix)).unwrap();
    match host_call("", "db", "del_prefixed", &request) {
        Ok(bytes) => deserialize::<u64>(&bytes).map_err(|_| DbError::DeserializationError),
        Err(e) => Err(DbError::Internal(e.to_string())),
    }
}
/// Set multiple key-value pairs.
pub fn set_many<T: Serialize>(table: &str, pairs: &(&str, &T)) -> Result<(), DbError> {
    unimplemented!()
}
