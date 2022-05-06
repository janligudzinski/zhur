use std::{net::SocketAddr, str::FromStr};

use common::prelude::{log::*, tokio};

use axum::{extract::Path, routing::any, Router};

async fn invoke(Path((owner, app, raw_path)): Path<(String, String, Option<String>)>) -> String {
    let raw_path = raw_path.unwrap_or("/".to_string());
    let res = format!("Invoking {}:{} with path of {}", owner, app, raw_path);
    info!("{}", &res);
    res
}

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
    let app = Router::new().route("/:owner/:app/*raw_path", any(invoke));
    let server = axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8000").unwrap())
        .serve(app.into_make_service());

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}
