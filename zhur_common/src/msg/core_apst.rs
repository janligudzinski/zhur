use crate::serde::{Deserialize, Serialize};

pub const DEFAULT_APST_ENDPOINT: &str = "tcp://127.0.0.1:8082";
#[derive(Clone, Debug, Deserialize, Serialize)]
/// This type represents requests for apps from the core.
pub struct Core2ApstReq {
    pub owner: String,
    pub app_name: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// This type represents replies to `Core2ApstReq`s.
pub enum Core2ApstRep {
    FoundCode(Vec<u8>),
    NoSuchApp
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// This type represents updates sent by the app store to the core of its own accord in a publish-subscribe pattern.
pub enum Apst2Core {
    /// An app was removed or deactivated; executors that have it loaded should be removed too.
    Remove,
    /// An app designated by the first pair of strings has been renamed; the new name is in the third string.
    Rename(String, String, String),
    /// An app designated by the pair of strings has been updated; find enclosed the new WASM code.
    Update(String, String, Vec<u8>)
}