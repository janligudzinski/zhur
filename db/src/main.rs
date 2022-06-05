use std::sync::Arc;

use common::db::*;
use common::errors::IpcError;
use common::prelude::*;
use log::*;
use tokio::net::UnixListener;

mod data;
use data::*;

const DB_SOCKET_PATH: &str = "/tmp/zhur-db.sck";
const DB_FILE_PATH: &str = "/tmp/zhur-db.sled";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_module_level("sled", log::LevelFilter::Warn)
        .init()
        .unwrap();
    // Open database.
    let db = sled::open(DB_FILE_PATH)?;
    let db = Arc::new(db);

    #[cfg(debug_assertions)] // only delete socket file in development use
    std::fs::remove_file(DB_SOCKET_PATH).ok();
    // Start listening for requests.
    let listener = UnixListener::bind(DB_SOCKET_PATH)?;
    while let Ok((conn, _)) = listener.accept().await {
        let db = db.clone();
        tokio::spawn(async move {
            let mut server = ipc::UnixServer::new(1024 * 16, conn);
            loop {
                match server.get_request::<DbRequest>().await {
                    Ok(req) => {
                        let response = match process_request(req, &db) {
                            Ok(r) => {
                                info!("Request processed successfully.");
                                r
                            }
                            Err(e) => {
                                warn!(
                                    "An error was encountered while processing the request: {}",
                                    &e
                                );
                                DbResponse::InternalError(e.to_string())
                            }
                        };
                        match server.send_response(&response).await {
                            Ok(_) => (),
                            Err(IpcError::ClientDisconnected) => {
                                error!("Client disconnected while trying to send response.");
                                break;
                            }
                            Err(e) => error!("{}", e),
                        }
                    }
                    Err(IpcError::ClientDisconnected) => {
                        info!("Client disconnected.");
                        break;
                    }
                    Err(e) => {
                        error!("{}", e);
                        break;
                    }
                };
            }
        });
    }
    Ok(())
}
