use zhur_apst::ApstServer;
use zhur_apst::DEFAULT_ENDPOINT;
use zhur_common::{init_logger, zmq::{Context, SocketType}};
use zhur_common::log::*;
fn main() {
    init_logger();
    let ctx = Context::new();
    let rep_socket = ctx.socket(SocketType::REP)
    .expect("Expected to be able to build a REP socket.");
    let endpoint = match std::env::var("ZHUR_APST_ENDPOINT") {
        Ok(e) => e,
        Err(_) => {
            warn!("ZHUR_APST_ENDPOINT not set. Assuming default of {:?}.", DEFAULT_ENDPOINT);
            DEFAULT_ENDPOINT.to_string()
        }
    };
    rep_socket.bind(&endpoint)
    .expect("Could not bind REP socket");
    let apst_server = ApstServer::new(rep_socket).run_as_thread();
    apst_server.join().unwrap();
}
