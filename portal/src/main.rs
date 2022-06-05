use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{
    routing::{delete, get, patch, post, put},
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
    simple_logger::SimpleLogger::new()
        .with_module_level("sled", log::LevelFilter::Warn)
        .with_module_level("hyper", log::LevelFilter::Warn)
        .with_module_level("mio", log::LevelFilter::Warn)
        .init()
        .unwrap();
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
        .route("/apps/:app_name", put(app_routes::upsert_app))
        .route("/apps/disable/:app_name", post(app_routes::disable_app))
        .route("/apps/enable/:app_name", post(app_routes::enable_app))
        .route("/apps/remove/:app_name", delete(app_routes::remove_app))
        .route(
            "/apps/rename/:old_name/:new_name",
            patch(app_routes::rename_app),
        )
        .layer(Extension(app_repo))
        .into_make_service();

    axum::Server::bind(&SocketAddr::from_str("0.0.0.0:8001").unwrap())
        .serve(router)
        .await
        .unwrap();
}
