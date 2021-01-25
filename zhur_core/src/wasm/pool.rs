use std::thread::JoinHandle;

use zhur_common::{bincode::{deserialize, serialize}, flume::{Receiver, Sender}, msg::{chan::Envelope, core_apst::{Core2ApstRep, Core2ApstReq}, core_kv::{Core2Kv, Kv2Core}}, zmq::Socket};
use zhur_common::log::*;
use zhur_invk::{Invocation, InvocationError};

use crate::wasm::InvocEnv;

use super::executor::Executor;
/// The "WASM executor pool" keeps track of WASM executors and distributes invocations among them.
pub struct WasmPool {
    /// This receiver handles incoming invocations to be passed out to executors.
    invoc_env_rx: Receiver<InvocEnv>,
    /// This is where invocations that can't be handled right away get put.
    outstanding_invocations: Vec<InvocEnv>,
    /// How many executors can be running at one time.
    max_executors: usize,
    /// The actual executors.
    executors: Vec<Executor>,
    /// The ZMQ socket used for requesting apps.
    apst_req_socket: Socket,
    kv_req_tx: Sender<Envelope<Core2Kv, Kv2Core>>
}
/// These are the possible decisions the `WasmPool` can make when receiving an invocation.
enum RunDecision {
    /// The invocation can't be run right away, as there are no free executors, and needs to be put on hold.
    PutAway,
    /// Forward the invocation to the executor at the given index, which already has the relevant code.
    Forward(usize),
    /// Forward the invocation to the executor at the given index, but make it load the relevant code first.
    Replace(usize),
    /// Spawn a new executor to handle this invocation.
    SpawnNew,
}

impl WasmPool {
    /// Gets the code for a given app.
    fn get_code(&self, owner: &str, app_name: &str) -> Result<Vec<u8>, InvocationError> {
        trace!("Requesting {}:{} code...", owner, app_name);
        let request = Core2ApstReq {
            owner: owner.to_string(),
            app_name: app_name.to_string()
        };
        let request_bytes = serialize(&request).unwrap();
        self.apst_req_socket.send(request_bytes, 0).unwrap();
        let reply_bytes = self.apst_req_socket.recv_bytes(0).unwrap();
        let reply = deserialize::<Core2ApstRep>(&reply_bytes).unwrap();
        match reply {
            Core2ApstRep::FoundCode(c) => {
                trace!("OK, found code for {}:{}", owner, app_name);
                Ok(c)
            },
            Core2ApstRep::NoSuchApp => {
                warn!("zhur_apst could not find code for {}:{}", owner, app_name);
                Err(InvocationError::NoSuchApp(owner.to_string(), app_name.to_string()))
            }
        }
    }
    /// Decides what to do with an incoming `Invocation`.
    fn decide(&self, i: &Invocation) -> RunDecision {
        // Do we have a free executor with the necessary app?
        for (index, each) in self.executors.iter().enumerate() {
            if each.owner == i.owner && each.app_name == i.app_name && each.free {
                return RunDecision::Forward(index);
            }
        }
        // If not, can we spawn another to handle it?
        if self.executors.len() < self.max_executors {
            return RunDecision::SpawnNew;
        }
        // No? Let's try evicting the first free executor we find, then.
        for (index, each) in self.executors.iter().enumerate() {
            if each.free {
                return RunDecision::Replace(index);
            }
        }
        // If there are no free executors and we can't spawn more, we can only wait.
        RunDecision::PutAway
    }
    /// Handles an incoming `Invocation`.
    fn handle(&mut self, env: InvocEnv) {
        // Check if any executors have become free.
        for each in self.executors.iter_mut() {
            each.check_status();
        }
        match self.decide(&env.0) {
            RunDecision::PutAway => {
                trace!("WasmPool could not handle invocation right away, putting away.");
                self.outstanding_invocations.push(env);
            }
            RunDecision::Forward(i) => {
                trace!("WasmPool found a free executor at #{}, invoking.", i);
                self.executors[i].invoke((env.0.payload, env.1));
            }
            RunDecision::SpawnNew => {
                trace!(
                    "WasmPool decided to spawn a new executor #{} for the current invocation.",
                    self.executors.len()
                );
                warn!("Not actually doing anything, zhur_apst hasn't been implemented yet.");
                let code = match self.get_code(&env.0.owner, &env.0.app_name) {
                    Ok(code) => code,
                    Err(e) => {
                        let e: Result<Vec<u8>, InvocationError> = Err(e);
                        let e_bytes = serialize(&e).unwrap();
                        warn!("Could not load code, sending back error.");
                        env.1.send(e_bytes).unwrap(); // Send back error result and return early.
                        return;
                    }
                };
                self.executors.push(Executor::new(
                    self.executors.len(),
                    env.0.owner.clone(),
                    env.0.app_name.clone(),
                    code,
                    self.kv_req_tx.clone()
                ));
                self.executors
                    .last_mut()
                    .unwrap()
                    .invoke((env.0.payload, env.1));
            }
            RunDecision::Replace(i) => {
                trace!("WasmPool decided to replace the code in executor #{} before using it to handle an invocation.", i);
                warn!("Not actually doing anything, zhur_apst hasn't been implemented yet.");
                let code = match self.get_code(&env.0.owner, &env.0.app_name) {
                    Ok(code) => code,
                    Err(e) => {
                        let e: Result<Vec<u8>, InvocationError> = Err(e);
                        let e_bytes = serialize(&e).unwrap();
                        warn!("Could not load code, sending back error.");
                        env.1.send(e_bytes).unwrap(); // Send back error result and return early.
                        return;
                    }
                };
                self.executors[i].load_code(env.0.owner.clone(), env.0.app_name.clone(), code);
                self.executors[i].owner = env.0.owner;
                self.executors[i].app_name = env.0.app_name;
                self.executors[i].invoke((env.0.payload, env.1));
            }
        }
    }
    pub fn new(max_executors: usize, invoc_env_rx: Receiver<InvocEnv>, apst_req_socket: Socket, kv_req_tx: Sender<Envelope<Core2Kv, Kv2Core>>) -> Self {
        Self {
            max_executors,
            invoc_env_rx,
            outstanding_invocations: Vec::new(),
            executors: Vec::new(),
            apst_req_socket,
            kv_req_tx
        }
    }
    /// Runs the `WasmPool` in a background thread.
    pub fn run_as_thread(self) -> JoinHandle<()> {
        std::thread::Builder::new()
            .name("wasm_pool".to_owned())
            .spawn(move || {
                let mut pool = self;
                loop {
                    let env = match pool.invoc_env_rx.recv() {
                        Ok(env) => env,
                        Err(_) => {
                            let text = "WasmPool could not receive incoming invocation envelope!";
                            error!("{}", text);
                            for each in pool.executors {
                                each.shutdown();
                            }
                            panic!("{}", text);
                        }
                    };
                    pool.handle(env);
                }
            })
            .expect("Could not launch WasmPool thread!")
    }
}
