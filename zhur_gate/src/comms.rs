use bincode::deserialize;
use zhur_common::{bincode, log::*, msg::chan::*, zmq};
use zhur_invk::{HttpRes, Invocation, InvocationError};
use zmq::{Context, Socket, SocketType};
/// Struct responsible for relaying requests from the gateway to the core.
pub struct Gate2CoreServer {
    /// ZMQ REQ socket.
    req_socket: Socket,
}
impl Gate2CoreServer {
    pub fn new(zmq_ctx: &Context) -> Self {
        Self {
            req_socket: {
                let sck = zmq_ctx
                    .socket(SocketType::REQ)
                    .expect("Expected to be able to construct a socket.");
                let endpoint = match std::env::var("ZHUR_CORE_REP_URI") {
                    Ok(s) => s,
                    Err(_) => {
                        warn!("ZHUR_CORE_REP_URI not set - assuming default value of tcp://127.0.0.1:8081!");
                        "tcp://127.0.0.1:8081".to_owned()
                    }
                };
                sck.connect(&endpoint).expect(
                    "Expected to be able to connect a REQ socket from the gateway to the core.",
                );
                sck
            },
        }
    }
}

impl HandleRequest<Invocation, Result<HttpRes, InvocationError>> for Gate2CoreServer {
    fn handle(&mut self, msg: Invocation) -> Result<HttpRes, InvocationError> {
        trace!("Handling an invocation...");
        let invoc_bytes = match bincode::serialize(&msg) {
            Ok(b) => b,
            Err(_) => return Err(InvocationError::SerializeErr),
        };
        trace!("Serialized invocation.");
        match self.req_socket.send(invoc_bytes, 0) {
            Ok(_) => (),
            Err(_) => return Err(InvocationError::NoCore),
        };
        trace!("Sent invocation to core module.");
        let response_bytes = match self.req_socket.recv_bytes(0) {
            Ok(b) => b,
            Err(_) => return Err(InvocationError::MalformedReply),
        };
        trace!("Received bytes back from core, deserializing to a Vec<u8> (which in turn should be a serialized HttpRes) or InvocationError...");
        let response = match deserialize::<Result<Vec<u8>, InvocationError>>(&response_bytes) {
            Ok(r) => r?,
            Err(_) => {
                warn!("Bytes received back from core were not a proper Result<Vec<u8>, InvocationError>");
                return Err(InvocationError::MalformedReply)
            }
        };
        trace!("Deserializing to HttpRes...");
        //dbg!(&response);
        match deserialize::<HttpRes>(&response) {
            Ok(r) => Ok(r),
            Err(_) => {
                warn!("The inner Vec<u8> could not be deserialized to a Result<HttpRes, InvocationError>!");
                Err(InvocationError::MalformedReply)
            }
        }
    }
}
