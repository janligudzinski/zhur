use std::sync::Arc;

use common::{
    db::{DbRequest, DbResponse},
    errors::{InvocationError, IpcError},
    invoke::{Invocation, InvocationResult},
    prelude::{
        tokio::{net::UnixStream, sync::mpsc::UnboundedSender, sync::Mutex},
        *,
    },
};
use engine::core::Core;
use log::*;
use tokio::net::UnixListener;
const ENGINE_SOCKET_PATH: &str = "/tmp/zhur-engine.sck";
const DB_SOCKET_PATH: &str = "/tmp/zhur-db.sck";

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
    let (db_tx, db_rx) = mpsc::unbounded_channel::<(DbRequest, UnboundedSender<DbResponse>)>();
    let db_rx = Arc::new(Mutex::new(db_rx));
    // Start core thread.
    Core::start_core_thread(code, inv_rx, db_tx, flags.owner, flags.name);
    // Start server.
    std::fs::remove_file(ENGINE_SOCKET_PATH).ok();
    let listener = UnixListener::bind(ENGINE_SOCKET_PATH)?;
    let db_conn = UnixStream::connect(DB_SOCKET_PATH).await?;
    let db_client = ipc::UnixClient::new(1024 * 16, db_conn);
    let db_client = Arc::new(Mutex::new(db_client));
    while let Ok((connection, _)) = listener.accept().await {
        let db_rx = db_rx.clone();
        let inv_tx = inv_tx.clone();
        let db_client = db_client.clone();
        tokio::spawn(async move {
            info!("Connection accepted.");
            let mut server = ipc::UnixServer::new(1024 * 8, connection);
            loop {
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
                let mut db_rx = db_rx.lock().await;
                // Use the select macro to handle requests from the core or return a response, whichever comes first.
                let response = loop {
                    tokio::select! {
                        // If the core thread has requested data from the DB.
                        db_req = db_rx.recv() => {
                            info!("Core thread has made a DB request.");
                            let (db_req, res_sender) = db_req.unwrap();
                            let mut db_client = db_client.lock().await;
                            let response: DbResponse = db_client.request(&db_req).await.unwrap();
                            info!("DB request was made.");
                            res_sender.send(response).unwrap();
                            info!("DB request sent to core thread.");
                        },
                        // If the core thread has returned a response:
                        response = inv_res_rx.recv() => {
                            let response = response.expect("Got a None response from the core thread.");
                            break response;
                        }
                    };
                };
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
