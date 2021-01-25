use std::{sync::{Arc, Mutex}, thread::JoinHandle};
use super::{ExecutorMsg};
use zhur_common::{bincode::{deserialize, serialize}, flume::{Receiver, Sender, unbounded}, msg::{chan::Envelope, core_kv::{Core2Kv, Kv2Core}}};
use zhur_common::log::*;
use zhur_invk::InvocationError;
use wapc::WapcHost;
use wasm3_provider::Wasm3EngineProvider;
use chrono::Utc;
/// Metadata accessible to the executor and to the WASM code running within it, neatly grouped.
#[derive(Clone)]
pub struct Metadata {
    pub owner: String,
    pub app_name: String,
    /// Inner executor's numeral ID. Inherited from the outer `Executor` struct.
    pub id: usize,
}

/// This struct contains the actual code engine used to run user-provided apps.
pub struct InnerExecutor {
    metadata: Arc<Mutex<Metadata>>, // This is so the host's closure can refer to the executor's metadata.
    // Since waPC execution is single-threaded and synchronous, we don't need an `RwLock`. In fact, when I tried to do that, I got poison errors.
    msg_rx: Receiver<ExecutorMsg>,
    done_tx: Sender<()>,
    host: WapcHost,
}
impl InnerExecutor {
    /// Creates an InnerExecutor in a thread. We can't first construct one and then run it as a thread because `WapcHost`s can't be moved between threads,
    /// so everything needs to be created in the new thread in one go.
    pub fn new(meta: Metadata, msg_rx: Receiver<ExecutorMsg>, done_tx: Sender<()>, initial_code: Vec<u8>, kv_req_tx: Sender<Envelope<Core2Kv, Kv2Core>>) -> JoinHandle<()> {
        std::thread::Builder::new()
        .name(format!("inner_executor_{}", meta.id))
        .spawn(move || {
            let meta_arc = Arc::new(Mutex::new(meta));
            let callback_meta = meta_arc.clone();
            trace!("Creating a new wasm engine...");
            let host = WapcHost::new(Box::new(Wasm3EngineProvider::new(&initial_code)),
            move |_id, _bd, _ns, op, payload| {
                let kv_tx = kv_req_tx.clone();
                let (kv_rep_tx, kv_rep_rx) = unbounded();
                trace!("Inner executor got host call for {:?}", op);
                match op {
                    "whoami" => {
                        let meta = callback_meta.lock().unwrap();
                        let strings = (meta.owner.clone(), meta.app_name.clone());
                        let bytes = serialize(&strings).unwrap();
                        Ok(bytes)
                    },
                    "kv_get" => {
                        let meta = callback_meta.lock().unwrap();
                        let owner = meta.owner.clone();
                        let (table, key) = deserialize::<(String, String)>(payload).unwrap();
                        let req = Core2Kv::KvGet(owner, table, key);
                        trace!("Requesting KvGet..");
                        kv_tx.send((req, kv_rep_tx)).unwrap();
                        let res = kv_rep_rx.recv().unwrap();
                        trace!("Requested KvGet.");
                        match res {
                            Kv2Core::Value(opt) => Ok(serialize(&opt).unwrap()),
                            _ => panic!("A KvGet returned something other than a value or lack thereof!")
                        }
                    },
                    "kv_set" => {
                        let meta = callback_meta.lock().unwrap();
                        let owner = meta.owner.clone();
                        let (table, key, value) = deserialize::<(String, String, Vec<u8>)>(payload).unwrap();
                        let req = Core2Kv::KvSet(owner, table, key, value);
                        kv_tx.send((req, kv_rep_tx)).unwrap();
                        let _res = kv_rep_rx.recv().unwrap();
                        dbg!(_res);
                        Ok(Vec::new())
                    },
                    "kv_del" => {
                        let meta = callback_meta.lock().unwrap();
                        let owner = meta.owner.clone();
                        let (table, key) = deserialize::<(String, String)>(payload).unwrap();
                        let req = Core2Kv::KvDel(owner, table, key);
                        kv_tx.send((req, kv_rep_tx)).unwrap();
                        let _res = kv_rep_rx.recv().unwrap();
                        dbg!(_res);
                        Ok(Vec::new())
                    },
                    "datetime" => {
                        let now = Utc::now().naive_utc();
                        let bytes = serialize(&now).unwrap();
                        Ok(bytes)
                    }
                    _ => {
                        Ok(Vec::new())
                    }
                }
            }).unwrap();
            trace!("Created a new wasm engine.");
            
            let exec = Self {
                metadata: meta_arc,
                msg_rx,
                done_tx,
                host,
            };

            loop {
                if !exec.handle() {
                    break;
                }
            }
        }).unwrap()
    }
    fn load_code(&self, owner: String, app_name: String, code: Vec<u8>) {
        let mut meta = self.metadata.lock().unwrap();
        info!(
            "Inner WASM executor #{} was requested to load the code for {}:{}.",
            meta.id,
            &owner,
            &app_name
        );
        meta.owner = owner;
        meta.app_name = app_name;
        match self.host.replace_module(&code) {
            Ok(_) => info!("Executor #{} successfully loaded the code for {}:{}.", meta.id, meta.owner, meta.app_name),
            Err(_e) => warn!("Executor #{} could not properly load the code for {}:{}!", meta.id, meta.owner, meta.app_name)
        }
    }
    /// Handles incoming `ExecutorMsg`s and decides whether or not the executor's loop should continue to run by returning a `bool`.
    fn handle(&self) -> bool {
        let meta = {
            let meta_lock = self.metadata.lock().unwrap();
            meta_lock.clone()
        };
        let msg = match self.msg_rx.recv() {
            Ok(msg) => msg,
            Err(_) => {
                let text = format!("Inner WASM executor #1 could not receive a message from its outer executor thread!");
                error!("{}", &text);
                panic!("{}", &text);
            }
        };
        match msg {
            ExecutorMsg::LoadCode(o, a, c) => {
                self.load_code(o, a, c);
            }
            ExecutorMsg::Invoke(env) => {
                let meta = {
                    let lock = self.metadata.lock().unwrap();
                    lock.clone()
                };
                trace!("Inner WASM executor #{} received an invocation.", meta.id);
                let output = self.host.call("handle_http", &env.0)
                .map_err(|e| {
                    InvocationError::WapcError(e.to_string())
                }); // the Ok value should be a serialized HttpRes
                let bytes = serialize(&output).expect("Serialization error in InnerExecutor");
                // Send output back.
                match env.1.send(bytes) {
                    Ok(_) => {
                        trace!("Inner WASM executor #{} responded!", meta.id);
                    }
                    Err(_) => {
                        let text = format!(
                            "Inner WASM executor #{} could not respond to the invocation it got!",
                            meta.id
                        );
                        error!("{}", &text);
                        panic!("{}", &text);
                    }
                }
                // Report we're done so the outer executor can mark itself as free.
                match self.done_tx.send(()) {
                    Ok(_) => {
                        trace!("Inner WASM executor #{} done!", meta.id);
                    }
                    Err(_) => {
                        let text = format!(
                            "Inner WASM executor #{} could not report a successful execution!",
                            meta.id
                        );
                        error!("{}", &text);
                        panic!("{}", &text);
                    }
                }
            },
            ExecutorMsg::Rename(a) => {
                let mut meta = self.metadata.lock().unwrap();
                meta.app_name = a;
            }
            ExecutorMsg::Shutdown => {
                trace!("Inner WASM executor #{} was told to shut down.", meta.id);
                return false; // quit thread loop
            }
        }
        true
    }
    
}