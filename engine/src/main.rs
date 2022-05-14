use common::{
    errors::{InvocationError, IpcError},
    invoke::{Invocation, InvocationResult},
    prelude::{tokio::sync::mpsc::UnboundedSender, *},
};
use engine::core::Core;
use log::*;
use tokio::net::UnixListener;
const ENGINE_SOCKET_PATH: &str = "/tmp/zhur-engine.sck";

use clap::Parser;
#[derive(Parser, Debug)]
struct Flags {
    #[clap(short, long)]
    owner: String,
    #[clap(short, long)]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;
    info!("Zhur engine started.");
    // Parse flags.
    let flags = Flags::parse();
    // Retrieve code.
    let code = match engine::code::get_code_simple(&flags.owner, &flags.name) {
        Some(c) => c,
        None => return Err(InvocationError::NoAppFound(flags.owner, flags.name).into()),
    };
    use tokio::sync::mpsc;

    let (inv_tx, inv_rx) =
        mpsc::unbounded_channel::<(Invocation, UnboundedSender<InvocationResult>)>();
    // Start core thread.
    Core::start_core_thread(code, inv_rx);
    // Start server.
    std::fs::remove_file(ENGINE_SOCKET_PATH).ok();
    let listener = UnixListener::bind(ENGINE_SOCKET_PATH)?;
    while let Ok((connection, _)) = listener.accept().await {
        let inv_tx = inv_tx.clone();
        tokio::spawn(async move {
            info!("Connection accepted.");
            let mut server = ipc::UnixServer::new(1024 * 8, connection);
            loop {
                let inv_tx = inv_tx.clone();
                let (inv_res_tx, mut inv_res_rx) = mpsc::unbounded_channel::<InvocationResult>();
                let invocation = match server.get_request::<Invocation>().await {
                    Ok(i) => i,
                    Err(IpcError::ClientDisconnected) => {
                        info!("Client disconnected.");
                        break;
                    }
                    Err(e) => {
                        error!("{}", e);
                        break;
                    }
                };
                // Send invocation to core thread.
                inv_tx
                    .send((invocation, inv_res_tx))
                    .expect("could not send an invocation to the core thread");
                // Await response.
                let response = inv_res_rx
                    .recv()
                    .await
                    .expect("Got a None response from the core thread.");
                match server.send_response(&response).await {
                    Ok(_) => (),
                    Err(e) => {
                        error!("{}", e);
                        break;
                    }
                }
            }
        });
    }
    std::fs::remove_file(ENGINE_SOCKET_PATH).ok();
    Ok(())
}
