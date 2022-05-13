use serde::{de::DeserializeOwned, Serialize};

use crate::error::DbError;

/// Create a new table.
pub fn create_table(table: &str) -> Result<(), DbError> {
    unimplemented!()
}
/// Check if a table exists.
pub fn table_exists(table: &str) -> Result<bool, DbError> {
    unimplemented!()
}
/// Delete a table.
pub fn delete_table(table: &str) -> Result<(), DbError> {
    unimplemented!()
}

/// Get a single value from a given table by its key.
pub fn get<T: DeserializeOwned>(table: &str, key: &str) -> Result<Option<T>, DbError> {
    unimplemented!()
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
