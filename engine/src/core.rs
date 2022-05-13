use std::sync::{Arc, Mutex};

use common::{
    errors::InvocationError,
    prelude::{log::warn, *},
};
use wapc::{WapcHost, WebAssemblyEngineProvider};
/// A Zhur core is a struct that handles executing incoming invocations through a waPC runtime.
pub struct Core {
    /// The waPC-compliant runtime.
    runtime: WapcHost,
    /// A wrapper for panic info text returned by guest apps.
    // This has to be wrapped in an Arc and a Mutex because we want to be able to modify this data from closures.
    panic_holder: Arc<Mutex<Option<String>>>,
}

impl Core {
    pub fn invoke_text(&mut self, payload: &str) -> Result<String, InvocationError> {
        let str_bytes =
            bincode::serialize(payload).map_err(|_| InvocationError::InvalidTextPayload)?;
        let result = self
            .runtime
            .call("text", &str_bytes)
            .map_err(|e| InvocationError::ExecutionError(e.to_string()));
        // Check if the guest program has panicked.
        if let Some(panic_info) = self.panic_info() {
            return Err(InvocationError::ExecutionError(panic_info));
        }
        let result = result?;
        let output = bincode::deserialize::<String>(&result)
            .map_err(|_| InvocationError::InvalidTextOutput)?;
        Ok(output)
    }
    pub fn new(engine: Box<dyn WebAssemblyEngineProvider>) -> Result<Self, InvocationError> {
        let panic_holder = Arc::new(Mutex::new(None));
        let callback_holder = panic_holder.clone();
        let host_callback = move |_id: u64, _bd: &str, _ns: &str, op: &str, pld: &[u8]| match op {
            "panic" => {
                let panic_string = std::str::from_utf8(pld)
                    .expect("Panic string in a guest app was not a valid UTF-8 string");
                warn!(
                    "A guest application has panicked with the panic info: {}",
                    panic_string
                );
                *callback_holder
                    .lock()
                    .expect("Could not lock panic string holder for writing.") =
                    Some(panic_string.to_owned());
                Ok(Vec::<u8>::new())
            }
            _ => unimplemented!("Errors for invalid host calls not implemented yet"),
        };
        let host = WapcHost::new(engine, Some(Box::new(host_callback)))
            .map_err(|e| InvocationError::HostInitializationError(e.to_string()))?;
        Ok(Self {
            runtime: host,
            panic_holder,
        })
    }
    /// Retrieves panic info after an invocation, if there was any.
    pub fn panic_info(&self) -> Option<String> {
        self.panic_holder
            .lock()
            .expect("Could not lock panic string holder for reading.")
            .clone()
    }
}
