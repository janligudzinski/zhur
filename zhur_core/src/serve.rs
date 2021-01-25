use std::thread::JoinHandle;

use crate::wasm::InvocEnv;
use zhur_common::{log::*, msg::{chan::Envelope, core_kv::{Core2Kv, DEFAULT_KV_ENDPOINT, Kv2Core}}};
use zhur_common::zmq::{Context, Socket, SocketType};
use zhur_common::{
    bincode::{deserialize, serialize},
    flume::{unbounded, Receiver, Sender},
};
use zhur_invk::{HttpRes, Invocation, InvocationError};
/// The ZMQ server that takes invocations incoming from the gateway and sends back bytes.
pub struct CoreServer {
    rep_socket: Socket,
    invoc_env_tx: Sender<InvocEnv>,
    reply_tx: Sender<Vec<u8>>,
    reply_rx: Receiver<Vec<u8>>,
}
impl CoreServer {
    pub fn new(zmq_ctx: &Context, invoc_env_tx: Sender<InvocEnv>) -> Self {
        let (reply_tx, reply_rx) = unbounded();
        Self {
            rep_socket: {
                let socket = zmq_ctx
                    .socket(SocketType::REP)
                    .expect("Expected to be able to construct a ZMQ socket.");
                let endpoint = match std::env::var("ZHUR_CORE_REP_URI") {
                    Ok(s) => s,
                    Err(_) => {
                        warn!("ZHUR_CORE_REP_URI not set - assuming default value of tcp://127.0.0.1:8081!");
                        "tcp://127.0.0.1:8081".to_owned()
                    }
                };
                socket
                    .bind(&endpoint)
                    .expect("Expected to be able to bind the core server socket.");
                socket
            },
            reply_tx,
            reply_rx,
            invoc_env_tx,
        }
    }
    pub fn handle(&self) {
        let bytes = match self.rep_socket.recv_bytes(0) {
            Ok(b) => {
                trace!("CoreServer received some bytes.");
                b
            }
            Err(_) => {
                panic!("CoreServer could not receive any bytes.")
            }
        };
        let response = self.handle_invoke_bytes(&bytes);
        // TODO: There is no need to deserialize into HttpReses or InvocationErrors within the core. The gateway does that anyway.
        //let res_bytes = &response).expect("Could not serialize a response.");

        match self.rep_socket.send(&response, 0) {
            Ok(_) => {
                trace!("Sent a response!");
                ()
            }
            Err(_) => panic!("Could not send a reply."),
        }
    }
    fn handle_invoke_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        let inv = match deserialize::<Invocation>(bytes) {
            Ok(i) => i,
            Err(_) => {
                warn!("The bytes we got could not be deserialized to an Invocation.");
                let err: Result<HttpRes, InvocationError> = Err(InvocationError::MalformedRequest);
                return serialize(&err).unwrap()
            }
        };
        trace!("Got an invocation for {}:{}", &inv.owner, &inv.app_name);
        let env = (inv, self.reply_tx.clone());
        self.invoc_env_tx
            .send(env)
            .expect("Expected to be able to send an invocation envelope from the CoreServer");
        let reply = self.reply_rx.recv().expect(
            "Expected to be able to receive a Vec<u8> back from the WasmPool in the CoreServer",
        );
        reply
    }
}

/// This ZMQ server handles K/V store requests.
pub struct KvServer {
    req_socket: Socket,
    kv_req_rx: Receiver<Envelope<Core2Kv, Kv2Core>>
}
impl KvServer {
    pub fn new(zmq_ctx: &Context, kv_req_rx: Receiver<Envelope<Core2Kv, Kv2Core>>) -> Self {
        let socket = zmq_ctx.socket(SocketType::REQ).unwrap();
        let endpoint = match std::env::var("ZHUR_KV_ENDPOINT") {
            Ok(e) => e,
            Err(_) => {
                warn!("ZHUR_KV_ENDPOINT not set. Assuming default of {:?}.", DEFAULT_KV_ENDPOINT);
                DEFAULT_KV_ENDPOINT.to_string()
            }
        };
        socket.connect(&endpoint).expect("Could not connect to KV server!");
        Self {
            req_socket: socket,
            kv_req_rx
        }
    }
    fn handle(&self) {
        let (request, return_tx) = self.kv_req_rx.recv().unwrap();
        trace!("Got Core2Kv request.");
        let req_bytes = serialize(&request).unwrap();
        self.req_socket.send(&req_bytes, 0).unwrap();
        trace!("Sent Core2Kv request to K/V store.");
        let res_bytes = self.req_socket.recv_bytes(0).unwrap();
        trace!("Got reply from K/V.");
        let response = deserialize(&res_bytes).unwrap();
        trace!("Deserialized to Kv2Core, sending back.");
        return_tx.send(response).unwrap();
    }
    pub fn run_as_thread(self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let server = self;
            loop {
                server.handle();
            }
        })
    }
}