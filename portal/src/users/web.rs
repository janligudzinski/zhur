use super::data::*;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use common::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    name: String,
    password: String,
}

pub async fn register(
    Extension(repo): Extension<Arc<UserRepo>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if repo.register_user(&req.name, &req.password) {
        (StatusCode::OK, Json("User successfully registered"))
    } else {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json("User not registered - the username was already taken."),
        )
    }
}
