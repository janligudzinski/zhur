use serde::{Deserialize, Serialize};

pub mod err;
pub use err::InvocationError;
pub mod http;
pub use http::*;
/// Struct representing a Zhur app invocation.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Invocation {
    /// The name of the user/org to whom the app belongs.
    pub owner: String,
    /// The name of the app itself.
    pub app_name: String,
    /// The input for the app.
    pub payload: Vec<u8>,
}
