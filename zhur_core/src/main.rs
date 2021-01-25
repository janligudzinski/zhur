use zhur_common::{init_logger, log::*, msg::core_apst::DEFAULT_APST_ENDPOINT, zmq::SocketType};
use zhur_common::{flume::unbounded, zmq::Context};
use zhur_core::{CoreServer, WasmPool, serve::KvServer};

fn main() {
    init_logger();
    let apst_endpoint = match std::env::var("ZHUR_APST_ENDPOINT") {
        Ok(e) => e,
        Err(_) => {
            warn!("ZHUR_APST_ENDPOINT not set. Assuming default of {:?}.", DEFAULT_APST_ENDPOINT);
            DEFAULT_APST_ENDPOINT.to_string()
        }
    };
    let zmq_ctx = Context::new();
    let apst_req_socket = zmq_ctx.socket(SocketType::REQ).unwrap();
    apst_req_socket.connect(&apst_endpoint).unwrap();
    let (invoc_env_tx, invoc_env_rx) = unbounded();
    let (kv_req_tx, kv_req_rx) = unbounded();
    let kv_server = KvServer::new(&zmq_ctx, kv_req_rx);
    kv_server.run_as_thread();
    let _wasm_pool = WasmPool::new(3, invoc_env_rx, apst_req_socket, kv_req_tx).run_as_thread();
    let server = CoreServer::new(&zmq_ctx, invoc_env_tx);
    loop {
        server.handle();
    }
}
