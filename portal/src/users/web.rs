use super::data::*;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use common::prelude::*;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
const HMAC_KEY: &[u8; 24] = b"DO NOT USE IN PRODUCTION";

#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginClaims {
    pub sub: String,
}
#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
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

pub async fn login(
    Extension(repo): Extension<Arc<UserRepo>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    if repo.login(&req.name, &req.password) {
        let claim = LoginClaims { sub: req.name };
        let key = Hmac::<Sha256>::new_from_slice(HMAC_KEY).unwrap();
        let token_str = claim.sign_with_key(&key).unwrap();
        Ok((StatusCode::OK, Json(token_str)))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json("Invalid username/password combination."),
        ))
    }
}