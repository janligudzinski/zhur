use std::sync::Arc;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use common::apst::{AppStoreRequest, AppStoreResponse};

use crate::users::web::LoginClaims;

use super::data::AppRepo;

pub async fn get_owned_apps(
    Extension(repo): Extension<Arc<AppRepo>>,
    claim: LoginClaims,
) -> impl IntoResponse {
    let mut client = repo.get_connection().await;
    let apps = match client
        .request(&AppStoreRequest::GetOwnedApps { owner: claim.sub })
        .await
        .unwrap()
    {
        AppStoreResponse::Apps { apps } => apps,
        e => panic!("Requested owned apps, got {:?}", e),
    };
    Json(apps)
}

pub async fn upsert_app(
    Path(app_name): Path<String>,
    Extension(repo): Extension<Arc<AppRepo>>,
    claim: LoginClaims,
    mut data: Multipart,
) -> impl IntoResponse {
    let mut client = repo.get_connection().await;
    let data = match data.next_field().await {
        Ok(Some(d)) => d,
        _ => return Err((StatusCode::BAD_REQUEST, "No code was sent.")),
    };
    let code = match data.bytes().await {
        Ok(b) => b,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "The code could not be extracted as bytes.",
            ))
        }
    }
    .to_vec();
    let request = AppStoreRequest::UpsertApp {
        owner: claim.sub.clone(),
        app_name,
        code,
    };
    match client.request(&request).await.unwrap() {
        AppStoreResponse::AppUpserted => Ok(format!("App \"{}\" upserted.", claim.sub)),
        e => panic!(
            "Upserted app, got a response other than AppUpserted: {:?}",
            e
        ),
    }
}

pub async fn remove_app(
    Path(app_name): Path<String>,
    Extension(repo): Extension<Arc<AppRepo>>,
    claim: LoginClaims,
) -> impl IntoResponse {
    let mut client = repo.get_connection().await;
    let req = AppStoreRequest::RemoveApp {
        owner: claim.sub,
        app_name,
    };
    match client.request(&req).await.unwrap() {
        AppStoreResponse::AppRemoved => (StatusCode::OK, "App removed"),
        e => panic!("Got a response other than AppRemoved: {:?}", e),
    }
}

pub async fn disable_app(
    Path(app_name): Path<String>,
    Extension(repo): Extension<Arc<AppRepo>>,
    claim: LoginClaims,
) -> impl IntoResponse {
    let mut client = repo.get_connection().await;
    let req = AppStoreRequest::DisableApp {
        owner: claim.sub,
        app_name,
    };
    match client.request(&req).await.unwrap() {
        AppStoreResponse::AppDisabled => (StatusCode::OK, "App disabled"),
        e => panic!("Got a response other than AppDisabled: {:?}", e),
    }
}

pub async fn enable_app(
    Path(app_name): Path<String>,
    Extension(repo): Extension<Arc<AppRepo>>,
    claim: LoginClaims,
) -> impl IntoResponse {
    let mut client = repo.get_connection().await;
    let req = AppStoreRequest::EnableApp {
        owner: claim.sub,
        app_name,
    };
    match client.request(&req).await.unwrap() {
        AppStoreResponse::AppEnabled => (StatusCode::OK, "App enabled"),
        e => panic!("Got a response other than AppEnabled: {:?}", e),
    }
}

pub async fn rename_app(
    Path((old_name, new_name)): Path<(String, String)>,
    Extension(repo): Extension<Arc<AppRepo>>,
    claim: LoginClaims,
) -> impl IntoResponse {
    let mut client = repo.get_connection().await;
    let req = AppStoreRequest::RenameApp {
        owner: claim.sub,
        old_name,
        new_name,
    };
    match client.request(&req).await.unwrap() {
        AppStoreResponse::AppRenamed(renamed) => {
            if renamed {
                (StatusCode::OK, "App renamed")
            } else {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "The new name is already taken.",
                )
            }
        }
        e => panic!("Got a response other than AppRenamed: {:?}", e),
    }
}
