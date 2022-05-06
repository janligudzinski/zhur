use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub mod http;

/// App invocation types. This decides how an app's response is processed.
#[derive(Deserialize, Serialize, Debug)]
pub enum Invocation {
    /// Simple invocation type we use for internal debugging.
    TextInvocation {
        ctx: InvocationContext,
        payload: String,
    },
}

/// Execution data concerning a particular invocation.
#[derive(Deserialize, Serialize, Debug)]
pub struct InvocationContext {
    /// Whose app we're invoking.
    pub owner: String,
    /// Which app it is.
    pub app: String,
    /// When the app was invoked.
    pub timestamp: DateTime<Utc>,
}
impl InvocationContext {
    pub fn new(owner: String, app: String) -> Self {
        Self {
            owner,
            app,
            timestamp: Utc::now(),
        }
    }
}

/// Responses to an invocation.
#[derive(Deserialize, Serialize, Debug)]
pub enum InvocationResponse {
    TextResponse {
        ctx: InvocationContext,
        payload: String,
    },
}
