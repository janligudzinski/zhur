use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
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
