use common::{errors::InvocationError, prelude::*};
use wapc::WapcHost;
/// A Zhur core is a struct that handles executing incoming invocations through a waPC runtime.
pub struct Core {
    /// The waPC-compliant runtime.
    runtime: WapcHost,
}

impl Core {
    pub fn invoke_text(&mut self, payload: &str) -> Result<String, InvocationError> {
        let str_bytes =
            bincode::serialize(payload).map_err(|_| InvocationError::InvalidTextPayload)?;
        let result = self
            .runtime
            .call("text", &str_bytes)
            .map_err(|e| InvocationError::ExecutionError(e.to_string()))?;
        let output = bincode::deserialize::<String>(&result)
            .map_err(|_| InvocationError::InvalidTextOutput)?;
        Ok(output)
    }
}
