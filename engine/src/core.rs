use common::{errors::InvocationError, prelude::*};
use wapc::{HostCallback, WapcHost, WebAssemblyEngineProvider};
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
    pub fn new(
        engine: Box<dyn WebAssemblyEngineProvider>,
        wasm_code: &[u8],
    ) -> Result<Self, InvocationError> {
        let host_callback =
            |_id: u64, _bd: &str, _ns: &str, op: &str, payload: &[u8]| Ok(Vec::<u8>::new());
        let host = WapcHost::new(engine, Some(Box::new(host_callback)))
            .map_err(|e| InvocationError::HostInitializationError(e.to_string()))?;
        host.replace_module(wasm_code)
            .map_err(|e| InvocationError::BadCode(e.to_string()))?;
        Ok(Self { runtime: host })
    }
}
