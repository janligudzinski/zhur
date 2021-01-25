use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use std::convert::Infallible;
use std::net::SocketAddr;

use zhur_common::log::*;

/// HTTP request handling code.
mod handle;
use handle::{handle_req, FullRequest};
/// Communication with the core module.
pub mod comms;
use zhur_common::msg::chan::*;
use zhur_invk::{HttpRes, Invocation, InvocationError};

/// Runs a Hyper HTTP server.
pub async fn start_server(client: ChannelClient<Invocation, Result<HttpRes, InvocationError>>) {
    let port = match std::env::var("ZHUR_GATE_PORT") {
        Ok(v) => match v.parse::<u16>() {
            Ok(n) => n,
            _ => {
                error!("ZHUR_GATE_PORT set to invalid value \"{}\", exiting.", &v);
                return;
            }
        },
        _ => {
            warn!("ZHUR_GATE_PORT env var not set. Assuming port 8080.");
            8080
        }
    };
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let ip = conn.remote_addr().to_string();
        let client = client.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_req(
                    FullRequest {
                        req,
                        ip: ip.clone(),
                    },
                    client.clone(), // TODO: remove this horrific hack
                )
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        error!("HTTP server error: {}", e);
    }
}
