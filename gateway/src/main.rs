use std::{net::SocketAddr, str::FromStr};

use common::{
    errors::InvocationError,
    invoke::{Invocation, InvocationContext, InvocationResponse, InvocationResult},
    prelude::{log::*, tokio},
};
use shared::http::*;

use axum::{
    body::Body, body::HttpBody, extract::Path, http::Request, response::IntoResponse, routing::any,
    Router,
};
use ipc::UnixClient;
mod conversion;
use conversion::*;

async fn invoke_http(owner: String, app: String, payload: HttpReq) -> anyhow::Result<HttpRes> {
    let stream = tokio::net::UnixStream::connect("/tmp/zhur-engine.sck").await?;
    let mut client = UnixClient::new(1024 * 8, stream);
    let invocation = Invocation::HttpInvocation {
        ctx: InvocationContext::new(owner, app),
        payload,
    };
    let response = client.request::<_, InvocationResult>(&invocation).await?;
    match response? {
        InvocationResponse::HttpResponse { ctx, payload } => {
            info!(
                "Got HTTP response from engine for {}:{}.",
                ctx.owner, ctx.app
            );
            Ok(payload)
        }
        InvocationResponse::TextResponse { ctx: _, payload: _ } => {
            error!("Got a text invocation response for an HTTP invocation!");
            Err(InvocationError::InvokeTypeMismatch.into())
        }
    }
}

async fn invoke_text(owner: String, app: String, payload: String) -> anyhow::Result<String> {
    let stream = tokio::net::UnixStream::connect("/tmp/zhur-engine.sck").await?;
    let mut client = UnixClient::new(1024 * 8, stream);
    let invocation = Invocation::TextInvocation {
        ctx: InvocationContext::new(owner, app),
        payload,
    };
    let response = client.request::<_, InvocationResult>(&invocation).await??;
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

async fn http_invoke_handler(
    Path((owner, app, raw_path)): Path<(String, String, Option<String>)>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let raw_path = raw_path.unwrap_or("/".to_string());
    let is_plaintext = match req.headers().get(&axum::http::header::CONTENT_TYPE) {
        Some(c) => !is_mimetype_binary(c.to_str().unwrap_or("application/octet-stream")),
        None => false,
    };
    let body = match req.body_mut().data().await {
        None => None,
        Some(res) => match res {
            Ok(b) => {
                let bytes = b.to_vec();
                if is_plaintext {
                    let text = String::from_utf8(bytes).expect("should be plaintext");
                    Some(shared::http::HttpBody::Text(text))
                } else {
                    Some(shared::http::HttpBody::Binary(bytes))
                }
            }
            Err(e) => {
                todo!()
            }
        },
    };
    let (parts, _) = req.into_parts();
    let mut parts = HttpReqParts::from(parts);
    parts.path = raw_path; // strip owner and app info
    let req = HttpReq { body, parts };
    match invoke_http(owner, app, req).await {
        Ok(http_res) => Ok(HttpResWrapper(http_res)),
        Err(e) => Err(e.to_string()),
    }
}

fn is_mimetype_binary(mimetype: &str) -> bool {
    use mime::Mime;
    match mimetype.parse::<Mime>() {
        Ok(m) => match m.type_() {
            mime::TEXT => false,
            mime::APPLICATION => match m.subtype() {
                mime::JAVASCRIPT | mime::JSON => false,
                _ => true,
            },
            _ => true,
        },
        Err(e) => {
            warn!(
                "Error occurred when parsing the MIME type of a request: {}",
                e
            );
            warn!("Assuming binary format.");
            true
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
    let app = Router::new()
        .route("/text/:owner/:app/*raw_path", any(text_invoke_handler))
        .route("/:owner/:app/*raw_path", any(http_invoke_handler));
    let server = axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8000").unwrap())
        .serve(app.into_make_service());

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}
