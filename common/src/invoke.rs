use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::InvocationError;

use self::http::{HttpReq, HttpRes};
pub mod http;

/// Common shorthand for `Result<InvocationResponse, InvocationError>`. This is the type returned by the Zhur engine.
pub type InvocationResult = Result<InvocationResponse, InvocationError>;

/// App invocation types. This decides how an app's response is processed.
#[derive(Deserialize, Serialize, Debug)]
pub enum Invocation {
    /// Simple invocation type we use for internal debugging.
    TextInvocation {
        ctx: InvocationContext,
        payload: String,
    },
    /// This is our main invocation type, containing an HTTP request.
    HttpInvocation {
        ctx: InvocationContext,
        payload: HttpReq,
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
    HttpResponse {
        ctx: InvocationContext,
        payload: HttpRes,
    },
}
