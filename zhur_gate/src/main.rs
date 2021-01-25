use zhur_common::{init_logger, msg::chan::client_server, zmq};
use zhur_gate::comms::Gate2CoreServer;
use zhur_gate::start_server;
#[tokio::main]
async fn main() {
    let zmq_ctx = zmq::Context::new();
    let gate_core_server = Gate2CoreServer::new(&zmq_ctx);
    let (client, server) = client_server(gate_core_server);
    std::thread::spawn(move || {
        let mut server = server;
        loop {
            server.handle()
        }
    });
    init_logger();
    start_server(client).await;
}
