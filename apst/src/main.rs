use std::sync::Arc;

use common::apst::*;
use common::errors::IpcError;
use common::prelude::tokio::net::UnixListener;
use common::prelude::*;
use ipc::UnixServer;
use log::*;

use crate::data::AppStore;
const STORE_SOCKET_PATH: &str = "/tmp/zhur-apst.sck";
const STORE_DB_PATH: &str = "/tmp/zhur-apst.sled";

mod data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init().unwrap();
    info!("Starting app store server.");
    info!("Opening database...");
    let db = sled::open(STORE_DB_PATH)?;
    let app_store = Arc::new(AppStore::new(db));

    #[cfg(debug_assertions)]
    std::fs::remove_file(STORE_SOCKET_PATH).ok();

    let listener = UnixListener::bind(STORE_SOCKET_PATH)?;
    while let Ok((stream, _)) = listener.accept().await {
        let app_store = app_store.clone();
        tokio::spawn(async move {
            let mut server = UnixServer::new(1024 * 1024 * 40, stream);
            loop {
                let req: AppStoreRequest = match server.get_request().await {
                    Ok(r) => r,
                    Err(IpcError::ClientDisconnected) => {
                        info!("Client disconnected.");
                        break;
                    }
                    Err(e) => {
                        warn!("Error while reading request to App Store: {}", e);
                        break;
                    }
                };
                let res = app_store.handle_request(req);
                match server.send_response(&res).await {
                    Ok(_) => continue,
                    Err(e) => {
                        error!("Error while sending response from App Store: {}", e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}
