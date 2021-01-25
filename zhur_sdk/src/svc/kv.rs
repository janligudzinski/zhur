use bincode::*;
use serde::{Serialize, de::DeserializeOwned};
use wapc_guest::host_call;
/// Gets a value from the key-value data store.
pub fn kv_get<T: DeserializeOwned>(table: &str, key: &str) -> Option<T> {
    let request = (table.to_string(), key.to_string());
    let req_bytes = serialize(&request).unwrap();
    let outer_res_bytes = host_call("", "", "kv_get", &req_bytes).unwrap();
    let res_bytes = deserialize::<Option<Vec<u8>>>(&outer_res_bytes).unwrap()?;
    Some(deserialize::<T>(&res_bytes).unwrap())
}
/// Sets a value in the key-value store.
pub fn kv_set<T: Serialize>(table: &str, key: &str, value: &T) {
    let val_bytes = serialize(&value).unwrap();
    let request = (table.to_string(), key.to_string(), val_bytes);
    let req_bytes = serialize(&request).unwrap();
    host_call("", "", "kv_set", &req_bytes).unwrap();
}
/// Deletes a value in the key-value store.
pub fn kv_del(table: &str, key: &str) {
    let request = (table.to_string(), key.to_string());
    let req_bytes = serialize(&request).unwrap();
    host_call("", "", "kv_del", &req_bytes).unwrap();
}