use std::{net::SocketAddr, str::FromStr};

use common::{
    errors::InvocationError,
    invoke::{Invocation, InvocationContext, InvocationResponse},
    prelude::{log::*, tokio},
};

use axum::{extract::Path, routing::any, Router};
use ipc::UnixClient;

async fn invoke_text(owner: String, app: String, payload: String) -> anyhow::Result<String> {
    let stream = tokio::net::UnixStream::connect("/tmp/zhur-engine.sck").await?;
    let mut client = UnixClient::new(1024 * 8, stream);
    let invocation = Invocation::TextInvocation {
        ctx: InvocationContext::new(owner, app),
        payload,
    };
    let response = client.request::<_, InvocationResponse>(&invocation).await?;
    match response {
        InvocationResponse::TextResponse { ctx: _, payload } => {
            info!("Got response from engine:\n{}", &payload);
            Ok(payload)
        }
        InvocationResponse::HttpResponse { ctx: _, payload: _ } => {
            error!("Got an HTTP invocation response for a text invocation!");
            Err(InvocationError::InvokeTypeMismatch.into())
        }
    }
}

async fn text_invoke_handler(
    Path((owner, app, raw_path)): Path<(String, String, Option<String>)>,
) -> String {
    let raw_path = raw_path.unwrap_or("/".to_string());
    let res = format!("Invoking {}:{} with payload of {}", owner, app, raw_path);
    info!("{}", &res);
    match invoke_text(owner, app, raw_path).await {
        Ok(x) => x,
        Err(e) => format!("Error: {e}"),
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
    let app = Router::new().route("/:owner/:app/*raw_path", any(text_invoke_handler));
    let server = axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8000").unwrap())
        .serve(app.into_make_service());

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}
