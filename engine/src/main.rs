use common::{
    errors::IpcError,
    invoke::{
        Invocation::{self, *},
        InvocationResponse::{self, *},
    },
    prelude::*,
};
use log::*;
use tokio::net::UnixListener;
const ENGINE_SOCKET_PATH: &str = "/tmp/zhur-engine.sck";

fn handle_invocation(invocation: Invocation) -> InvocationResponse {
    match invocation {
        TextInvocation { ctx, payload } => {
            let hello_world = format!(
                "Hello {}, this is {}'s app named {} invoked at {}.",
                payload, &ctx.owner, &ctx.app, &ctx.timestamp
            );
            TextResponse {
                ctx,
                payload: hello_world,
            }
        }
        _ => panic!("HTTP invocations not supported yet!"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;
    info!("Zhur engine started.");
    std::fs::remove_file(ENGINE_SOCKET_PATH).ok();
    let listener = UnixListener::bind(ENGINE_SOCKET_PATH)?;
    while let Ok((connection, _)) = listener.accept().await {
        tokio::spawn(async move {
            info!("Connection accepted.");
            let mut server = ipc::UnixServer::new(1024 * 8, connection);
            loop {
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
                let response = handle_invocation(invocation);
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
