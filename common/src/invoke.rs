use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// App invocation types. This decides how an app's response is processed.
#[derive(Deserialize, Serialize, Debug)]
pub enum InvocationType {
    /// (Re)serialize whatever the app returns as JSON.
    Json,
}

/// Execution data concerning a particular invocation.
#[derive(Deserialize, Serialize)]
pub struct InvocationContext {
    /// Whose app we're invoking.
    pub owner: String,
    /// Which app it is.
    pub app: String,
    /// When the app was invoked.
    pub timestamp: DateTime<Utc>,
    /// What type of invocation we're dealing with.
    inv_type: InvocationType,
}
impl InvocationContext {
    pub fn new(owner: String, app: String, inv_type: InvocationType) -> Self {
        Self {
            owner,
            app,
            timestamp: Utc::now(),
            inv_type,
        }
    }
}
/// An invocation sent to the app engine, containing an arbitrary serializable payload.
#[derive(Deserialize, Serialize)]
pub struct Invocation {
    pub ctx: InvocationContext,
    pub payload: Vec<u8>,
}
impl Invocation {
    pub fn new<P: Serialize>(ctx: InvocationContext, payload: &P) -> Self {
        Self {
            ctx,
            payload: bincode::serialize(payload).expect("Could not serialize invocation payload."),
        }
    }
}
/// An arbitrary JSON response from the app engine. Currently the only supported type.
#[derive(Deserialize, Serialize)]
pub struct JsonResponse {
    pub ctx: InvocationContext,
    pub payload: String,
}
