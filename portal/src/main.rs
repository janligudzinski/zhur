use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{routing::post, Extension};
use common::prelude::*;
mod users;
use users::{data::UserRepo, web as user_routes};
const USERS_DB_PATH: &str = "/tmp/zhur-users.sled";
#[tokio::main]
async fn main() {
    let db = sled::open(USERS_DB_PATH).unwrap();
    let repo = Arc::new(UserRepo::new(db));
    let router = axum::Router::new()
        .route("/register", post(user_routes::register))
        .route("/login", post(user_routes::login))
        .layer(Extension(repo))
        .into_make_service();

    axum::Server::bind(&SocketAddr::from_str("0.0.0.0:8001").unwrap())
        .serve(router)
        .await
        .unwrap();
}
