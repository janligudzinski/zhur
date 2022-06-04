use common::apst::*;
use common::errors::IpcError;
use common::prelude::tokio::net::UnixListener;
use common::prelude::*;
use ipc::UnixServer;
use log::*;
const STORE_SOCKET_PATH: &str = "/tmp/zhur-apst.sck";
const STORE_DB_PATH: &str = "/tmp/zhur-apst.sled";

mod data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    info!("Starting app store server.");
    info!("Opening database...");
    let db = sled::open(STORE_DB_PATH)?;
    let listener = UnixListener::bind(STORE_SOCKET_PATH)?;
    while let Ok((stream, _)) = listener.accept().await {
        let mut server = UnixServer::new(1024 * 1024 * 20, stream);
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
        }
    }
    Ok(())
}
