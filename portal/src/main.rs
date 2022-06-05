use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{
    routing::{get, post},
    Extension,
};
use common::prelude::*;
mod users;
use users::{data::UserRepo, web as user_routes};
mod apst;
use apst::{data::AppRepo, web as app_routes};
const USERS_DB_PATH: &str = "/tmp/zhur-users.sled";
const STORE_SOCKET_PATH: &str = "/tmp/zhur-apst.sck";
#[tokio::main]
async fn main() {
    let db = sled::open(USERS_DB_PATH).unwrap();
    let app_repo = Arc::new(AppRepo::new(STORE_SOCKET_PATH));
    let user_repo = Arc::new(UserRepo::new(db));
    let router = axum::Router::new()
        .route("/register", post(user_routes::register))
        .route("/login", post(user_routes::login))
        .route("/whoami", post(user_routes::whoami))
        .route("/change_password", post(user_routes::change_password))
        .layer(Extension(user_repo))
        .route("/apps", get(app_routes::get_owned_apps))
        .layer(Extension(app_repo))
        .into_make_service();

    axum::Server::bind(&SocketAddr::from_str("0.0.0.0:8001").unwrap())
        .serve(router)
        .await
        .unwrap();
}
