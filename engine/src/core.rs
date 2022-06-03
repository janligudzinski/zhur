use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use common::{
    db::{DbRequest, DbResponse},
    errors::InvocationError,
    invoke::{Invocation, InvocationResult},
    prelude::{
        bincode::{deserialize, serialize},
        chrono,
        log::{info, warn},
        tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        *,
    },
};
use shared::http::{HttpReq, HttpRes};
use wapc::{WapcHost, WebAssemblyEngineProvider};
use wasm3_provider::Wasm3EngineProvider;

use crate::invoke::handle_invocation;
/// A Zhur core is a struct that handles executing incoming invocations through a waPC runtime.
pub struct Core {
    /// The waPC-compliant runtime.
    runtime: WapcHost,
    /// A wrapper for panic info text returned by guest apps.
    // This has to be wrapped in an Arc and a Mutex because we want to be able to modify this data from closures.
    panic_holder: Arc<Mutex<Option<String>>>,
}

impl Core {
    pub fn invoke_http(&mut self, payload: &HttpReq) -> Result<HttpRes, InvocationError> {
        let req_bytes = bincode::serialize(payload).map_err(|_| InvocationError::BadHttpRequest)?;
        let result = self
            .runtime
            .call("http", &req_bytes)
            .map_err(|e| InvocationError::ExecutionError(e.to_string()));
        // Check if the guest program has panicked.
        if let Some(panic_info) = self.panic_info() {
            return Err(InvocationError::ExecutionError(panic_info));
        }
        let result = result?;
        let output = bincode::deserialize::<HttpRes>(&result)
            .map_err(|_| InvocationError::InvalidTextOutput)?;
        Ok(output)
    }
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
    pub fn new(
        engine: Box<dyn WebAssemblyEngineProvider>,
        db_tx: UnboundedSender<(DbRequest, UnboundedSender<DbResponse>)>,
        owner: String,
        app_name: String,
    ) -> Result<Self, InvocationError> {
        let panic_holder = Arc::new(Mutex::new(None));
        let callback_holder = panic_holder.clone();
        let host_callback = move |_id: u64, _bd: &str, ns: &str, op: &str, pld: &[u8]| {
            let db_tx = db_tx.clone();
            match ns {
                "internals" => match op {
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
                    "whoami" => {
                        let response_bytes = serialize(&(&owner, &app_name)).unwrap();
                        Ok(response_bytes)
                    }
                    _ => unimplemented!("Errors for invalid host calls not implemented yet"),
                },
                "datetime" => match op {
                    "now" => {
                        let naive_dt = chrono::Utc::now().naive_utc();
                        Ok(bincode::serialize(&naive_dt).unwrap())
                    }
                    _ => unimplemented!("Errors for invalid host calls not implemented yet"),
                },
                "db" => {
                    let response = Self::process_db_request(owner.clone(), op, pld, db_tx);
                    Ok(bincode::serialize(&response).unwrap())
                }
                _ => unimplemented!("Errors for invalid host calls not implemented yet"),
            }
        };

        let host = WapcHost::new(engine, Some(Box::new(host_callback)))
            .map_err(|e| InvocationError::HostInitializationError(e.to_string()))?;
        Ok(Self {
            runtime: host,
            panic_holder,
        })
    }
    fn process_db_request(
        owner: String,
        op: &str,
        pld: &[u8],
        db_tx: UnboundedSender<(DbRequest, UnboundedSender<DbResponse>)>,
    ) -> DbResponse {
        let (res_tx, mut res_rx) = unbounded_channel::<DbResponse>();
        let request = match op {
            "get" => {
                let (table, key) = deserialize::<(String, String)>(pld).unwrap();
                DbRequest::Get { owner, table, key }
            }
            "set" => {
                let (table, key, value) = deserialize::<(String, String, Vec<u8>)>(pld).unwrap();
                DbRequest::Set {
                    owner,
                    table,
                    key,
                    value,
                }
            }
            "del" => {
                let (table, key) = deserialize::<(String, String)>(pld).unwrap();
                DbRequest::Del { owner, table, key }
            }
            "get_prefixed" => {
                let (table, prefix) = deserialize::<(String, String)>(pld).unwrap();
                DbRequest::GetPrefixed {
                    owner,
                    table,
                    prefix,
                }
            }
            "del_prefixed" => {
                let (table, prefix) = deserialize::<(String, String)>(pld).unwrap();
                DbRequest::DelPrefixed {
                    owner,
                    table,
                    prefix,
                }
            }
            "set_many" => {
                let (table, pairs) = deserialize::<(String, Vec<(String, Vec<u8>)>)>(pld).unwrap();
                DbRequest::SetMany {
                    owner,
                    table,
                    pairs,
                }
            }
            other => panic!("An unsupported db operation was attempted: {}", other),
        };
        db_tx.send((request, res_tx)).unwrap();
        res_rx.blocking_recv().unwrap()
    }
    /// Retrieves panic info after an invocation, if there was any.
    pub fn panic_info(&self) -> Option<String> {
        self.panic_holder
            .lock()
            .expect("Could not lock panic string holder for reading.")
            .clone()
    }
    /// Starts a thread on which a core is created and responds to incoming invocations.
    pub fn start_core_thread(
        code: Vec<u8>,
        mut inv_rx: UnboundedReceiver<(Invocation, UnboundedSender<InvocationResult>)>,
        db_tx: UnboundedSender<(DbRequest, UnboundedSender<DbResponse>)>,
        owner: String,
        app_name: String,
    ) {
        std::thread::spawn(move || {
            info!("Core thread starting.");
            let provider = Wasm3EngineProvider::new(&code);
            let mut core = Self::new(Box::new(provider), db_tx, owner, app_name).unwrap();
            loop {
                let (invocation, res_tx) = match inv_rx.blocking_recv() {
                    Some(r) => r,
                    None => {
                        warn!("Could not recv() an invocation on the core thread, exiting loop.");
                        break;
                    }
                };
                let response = handle_invocation(invocation, &mut core);
                res_tx
                    .send(response)
                    .expect("Could not send a response from the core thread.");
            }
            info!("Core thread has ended operation.");
        });
    }
}
