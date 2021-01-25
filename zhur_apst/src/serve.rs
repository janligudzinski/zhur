use bincode::{deserialize, serialize};
use zhur_common::bincode;
use zhur_common::msg::core_apst::{Core2ApstRep, Core2ApstReq};
use zhur_common::zmq::Socket;
use zhur_common::log::*;
const ECHO_CODE: &[u8] = include_bytes!("../../examples/target/wasm32-unknown-unknown/release/echo.wasm");
const COUNTER_CODE: &[u8] = include_bytes!("../../examples/target/wasm32-unknown-unknown/release/counter.wasm");
const TODOS_CODE: &[u8] = include_bytes!("../../examples/target/wasm32-unknown-unknown/release/todos.wasm");
pub struct ApstServer {
    /// ZMQ rep socket for handling incoming requests.
    rep_socket: Socket,
}

impl ApstServer {
    pub fn new(rep_socket: Socket) -> Self {
        Self { rep_socket }
    }
    /// Handles `Core2Apst` requests.
    fn handle_c2a_requests(&self) {
        let request_bytes = self.rep_socket.recv_bytes(0)
        .expect("Expected to be able to receive a request!");
        let request = deserialize::<Core2ApstReq>(&request_bytes)
        .expect("Expected to be able to deserialize request as Core2ApstReq");
        trace!("Got a request for the code for {}:{}!", &request.owner, &request.app_name);
        let reply = self.handle_core2apst(&request);
        let reply_bytes = serialize::<Core2ApstRep>(&reply)
        .expect("Expected to serialize into Core2ApstRep");
        trace!("Created a response...");
        self.rep_socket.send(reply_bytes, 0)
        .expect("Expected to send a reply back to core.");
        trace!("Response sent!");
    }
    fn handle_core2apst(&self, req: &Core2ApstReq) -> Core2ApstRep {
        match req.owner.as_str() {
            "zhur" => match req.app_name.as_str() {
                "echo" => {
                    trace!("Found the requested code for {}:{}!", &req.owner, &req.app_name);
                    Core2ApstRep::FoundCode(ECHO_CODE.to_vec())        
                },
                "counter" => {
                    trace!("Found the requested code for {}:{}!", &req.owner, &req.app_name);
                    Core2ApstRep::FoundCode(COUNTER_CODE.to_vec())        
                },
                "todos" => {
                    trace!("Found the requested code for {}:{}!", &req.owner, &req.app_name);
                    Core2ApstRep::FoundCode(TODOS_CODE.to_vec())     
                },
                _ => {
                    warn!("Did not find the requested code for {}:{}!", &req.owner, &req.app_name);
                    Core2ApstRep::NoSuchApp
                }
            },
            _ => {
                warn!("Did not find the requested code for {}:{}!", &req.owner, &req.app_name);
                Core2ApstRep::NoSuchApp
            }
        }
    }
    pub fn run_as_thread(self) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let server = self;
            loop {
                server.handle_c2a_requests();
            }
        })
    }
}