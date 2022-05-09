use std::sync::{Arc, RwLock};

use common::{
    errors::{InvocationError, IpcError},
    invoke::{
        Invocation::{self, *},
        InvocationResponse::{self, *},
        InvocationResult,
    },
    prelude::{tokio::sync::mpsc::UnboundedSender, *},
};
use engine::invoke::*;
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

    let (inv_tx, mut inv_rx) =
        mpsc::unbounded_channel::<(Invocation, UnboundedSender<InvocationResult>)>();
    std::thread::spawn(move || {
        info!("Core thread starting.");
        // Create core.
        let provider = wasm3_provider::Wasm3EngineProvider::new(&code);
        let mut core = engine::core::Core::new(Box::new(provider)).unwrap();
        loop {
            let (invocation, res_tx) = match inv_rx.blocking_recv() {
                Some(r) => r,
                None => {
                    warn!("Could not recv() an invocation on the core thread, exiting loop.");
                    break;
                }
            };
            let response = handle_invocation(invocation, &mut core);
            res_tx
                .send(response)
                .expect("Could not send a response from the core thread.");
        }
        info!("Core thread has ended operation.");
    });
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
