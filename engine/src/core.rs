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
        tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender},
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
        let db_holder = Arc::new(Mutex::new(BTreeMap::<String, Vec<u8>>::new()));
        let db = db_holder.clone();
        let host_callback = move |_id: u64, _bd: &str, ns: &str, op: &str, pld: &[u8]| match ns {
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
            "db" => match op {
                "get" => {
                    let (table, key) = deserialize::<(&str, &str)>(pld).unwrap();
                    let full_key = format!("{}:{}", table, key);
                    let value = { db.lock().unwrap().get(&full_key).map(|r| r.to_owned()) };
                    let answer = serialize(&value.to_owned()).unwrap();
                    Ok(answer)
                }
                "set" => {
                    let (table, key, value) = deserialize::<(&str, &str, Vec<u8>)>(pld).unwrap();
                    let full_key = format!("{}:{}", table, key);
                    {
                        db.lock().unwrap().insert(full_key, value);
                    }
                    Ok(vec![])
                }
                "del" => {
                    let (table, key) = deserialize::<(&str, &str)>(pld).unwrap();
                    let full_key = format!("{}:{}", table, key);
                    db.lock().unwrap().remove(&full_key);
                    Ok(vec![])
                }
                "get_prefixed" => {
                    let (table, key_prefix) = deserialize::<(&str, &str)>(pld).unwrap();
                    let full_key_prefix = format!("{}:{}", table, key_prefix);
                    let mut values_found = vec![];
                    for (_key, value) in db
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|(key, _)| key.starts_with(&full_key_prefix))
                    {
                        values_found.push(value.clone());
                    }
                    let answer = serialize(&values_found).unwrap();
                    Ok(answer)
                }
                "del_prefixed" => {
                    let (table, key_prefix) = deserialize::<(&str, &str)>(pld).unwrap();
                    let full_key_prefix = format!("{}:{}", table, key_prefix);
                    let mut del_counter = 0u64;
                    let mut db = db.lock().unwrap();
                    let keys = {
                        db.keys()
                            .filter(|key| key.starts_with(&full_key_prefix))
                            .map(|key| key.to_owned())
                            .collect::<Vec<_>>()
                    };
                    for key in keys {
                        del_counter += 1;
                        db.remove(&key);
                    }
                    Ok(serialize(&del_counter).unwrap())
                }
                "set_many" => {
                    let (table, pairs) = deserialize::<(&str, Vec<(&str, Vec<u8>)>)>(pld).unwrap();
                    let mut db = db.lock().unwrap();
                    for pair in pairs {
                        let full_key = format!("{}:{}", table, pair.0);
                        db.insert(full_key, pair.1);
                    }
                    Ok(vec![])
                }
                _ => unimplemented!("Errors for invalid host calls not implemented yet"),
            },
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
