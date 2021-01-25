use zhur_common::{bincode::{deserialize, serialize}, init_logger, msg::core_kv::{Core2Kv, DEFAULT_KV_ENDPOINT, Kv2Core}, zmq::{Context, SocketType}};
use zhur_common::log::*;
fn main() {
    init_logger();
    let db = sled::open("~/.zhur/kv.sled").unwrap();
    let zmq_ctx = Context::new();
    let socket = zmq_ctx.socket(SocketType::REP).unwrap();
    let endpoint = match std::env::var("ZHUR_KV_ENDPOINT") {
        Ok(e) => e,
        Err(_) => {
            warn!("ZHUR_KV_ENDPOINT not set. Assuming default of {:?}.", DEFAULT_KV_ENDPOINT);
            DEFAULT_KV_ENDPOINT.to_string()
        }
    };
    socket.bind(&endpoint).expect("Could not connect to KV server!");
    loop {
        let request_bytes = socket.recv_bytes(0).unwrap();
        trace!("Got request bytes.");
        let request: Core2Kv = deserialize(&request_bytes).unwrap();
        let response = match request {
            Core2Kv::KvGet(owner, table, key) => {
                let full_key = format!("{}:{}:{}", owner, table, key);
                trace!("Got a request to get {}", &full_key);
                let bytes = db.get(&full_key).unwrap().map(|ivec| ivec.to_vec());
                Kv2Core::Value(bytes)
            },
            Core2Kv::KvSet(owner, table, key, value) => {
                let full_key = format!("{}:{}:{}", owner, table, key);
                trace!("Got a request to set {}", &full_key);
                db.insert(&full_key, value).unwrap();
                Kv2Core::OperationSuccessful
            },
            Core2Kv::KvDel(owner, table, key) => {
                let full_key = format!("{}:{}:{}", owner, table, key);
                trace!("Got a request to delete {}", &full_key);
                db.remove(&full_key).unwrap();
                Kv2Core::OperationSuccessful
            }
        };
        let res_bytes = serialize(&response).unwrap();
        trace!("Sending reply...");
        socket.send(res_bytes, 0).unwrap();
        trace!("Sent reply!");
    }
}
