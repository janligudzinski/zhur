use std::{net::SocketAddr, str::FromStr};

use common::prelude::{log::*, tokio};

use axum::{extract::Path, routing::any, Router};

async fn invoke(Path((owner, app)): Path<(String, String)>) -> String {
    let res = format!("Invoking {}:{}", owner, app);
    info!("{}", &res);
    res
}

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
    let app = Router::new().route("/:owner/:app", any(invoke));
    let server = axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8000").unwrap())
        .serve(app.into_make_service());

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}
