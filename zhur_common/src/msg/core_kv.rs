use crate::serde::{Deserialize, Serialize};

pub const DEFAULT_KV_ENDPOINT: &str = "tcp://127.0.0.1:8085";

/// This type represents requests made by the core to the KV store.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Core2Kv {
    /// Get owner:table:key.
    KvGet(String, String, String),
    /// Set owner:table:key:value.
    KvSet(String, String, String, Vec<u8>),
    /// Delete owner:table:key.
    KvDel(String, String, String)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Kv2Core {
    Value(Option<Vec<u8>>),
    OperationSuccessful,
}