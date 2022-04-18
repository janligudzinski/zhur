use std::collections::HashMap;

use common::{
    invoke::{Invocation, JsonResponse},
    prelude::*,
};
use log::*;
use tokio::net::UnixListener;
const ENGINE_SOCKET_PATH: &str = "/tmp/zhur-engine.sck";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;
    info!("Zhur engine started.");
    std::fs::remove_file(ENGINE_SOCKET_PATH).ok();
    let listener = UnixListener::bind(ENGINE_SOCKET_PATH)?;
    while let Ok((connection, _)) = listener.accept().await {
        tokio::spawn(async move {
            info!("Connection accepted.");
            let mut server = common::ipc::UnixServer::new(1024 * 8, connection);
            loop {
                let invocation = match server.get_request::<Invocation>().await {
                    Ok(i) => i,
                    Err(e) => {
                        error!("{}", e);
                        break;
                    }
                };
                let hello_string: String =
                    bincode::deserialize(&invocation.payload).expect("deser");
                let hello_world = format!(
                    "Hello, {}, this is {}'s app named {} invoked at {}.",
                    hello_string,
                    invocation.ctx.owner,
                    invocation.ctx.app,
                    invocation.ctx.timestamp,
                );
                let mut skeleton = HashMap::new();
                skeleton.insert("message", hello_world);
                let response = JsonResponse {
                    ctx: invocation.ctx,
                    payload: serde_json::to_string(&skeleton).expect("error serializing hashmap"),
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
