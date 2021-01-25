use std::thread::JoinHandle;

use inner::{Metadata, InnerExecutor};
use zhur_common::{flume::{unbounded, Receiver, Sender}, log::*, msg::{chan::Envelope, core_kv::{Core2Kv, Kv2Core}}};

use super::PayloadEnv;
/// The inner execution logic.
mod inner;
/// Service logic.
pub mod svc;
pub struct Executor {
    /// The identifying number of the executor.
    pub id: usize,
    /// The username of this app's owner.
    pub owner: String,
    /// The name of the app currently held in the executor.
    pub app_name: String,
    /// Whether or not the executor can handle an invocation right now.
    pub free: bool,
    /// Sender for passing messages to the executor's `inner_thread`.
    msg_tx: Sender<ExecutorMsg>,
    /// Receiver for "done" messages from the inner thread.
    done_rx: Receiver<()>,
    /// The thread that holds the actual code engine.
    inner_thread: JoinHandle<()>,
}
/// Messages sent by the outer executor to its inner thread.
pub enum ExecutorMsg {
    /// Replace the WASM app currently loaded. Issued on app updates or substitutions.
    LoadCode(String, String, Vec<u8>),
    /// Issued on app renames.
    Rename(String),
    /// Self-explanatory. We're only passing in a payload and expecting a serialized reply which the core won't need to deserialize, thus we use `Vec<u8>` rather than complex types.
    Invoke(PayloadEnv),
    /// Shut the inner thread down.
    Shutdown,
}

impl Executor {
    pub fn check_status(&mut self) {
        match self.done_rx.try_recv() {
            Ok(_) => self.free = true,
            Err(_) => (),
        }
    }
    pub fn load_code(&self, owner: String, app_name: String, code: Vec<u8>) {
        self.msg_tx
            .send(ExecutorMsg::LoadCode(owner, app_name, code))
            .expect("Could not pass WASM code down to inner executor.");
    }
    pub fn invoke(&mut self, envelope: PayloadEnv) {
        self.free = false;
        self.msg_tx
            .send(ExecutorMsg::Invoke(envelope))
            .expect("Could not pass invocation down to inner executor.");
    }
    pub fn rename(&mut self, app_name: String) {
        self.app_name = app_name.clone();
        self.msg_tx
        .send(ExecutorMsg::Rename(app_name))
        .expect("Could not pass rename msg down to inner executor.");
    }
    pub fn shutdown(self) {
        match self.msg_tx.send(ExecutorMsg::Shutdown) {
            Ok(_) => (),
            Err(_) => {
                let text = format!(
                    "Executor #{} could not send a shutdown message to its inner thread!",
                    self.id
                );
                error!("{}", &text);
                panic!("{}", &text);
            }
        }
        match self.inner_thread.join() {
            Ok(_) => (),
            Err(_) => {
                let text = format!("Executor #{} could not join on its inner thread!", self.id);
                error!("{}", &text);
                panic!("{}", &text);
            }
        }
    }
    pub fn new(id: usize, owner: String, app_name: String, initial_code: Vec<u8>, kv_req_tx: Sender<Envelope<Core2Kv, Kv2Core>>) -> Self {
        let (msg_tx, msg_rx) = unbounded();
        let (done_tx, done_rx) = unbounded();
        let meta = Metadata {
            owner: owner.clone(),
            app_name: app_name.clone(),
            id
        };
        let inner = InnerExecutor::new(meta, msg_rx, done_tx, initial_code, kv_req_tx);
        Self {
            inner_thread: inner,
            owner,
            app_name,
            free: true,
            done_rx,
            msg_tx,
            id
        }
    }
}
