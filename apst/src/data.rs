use common::apst::{AppStoreRequest, AppStoreResponse};
use sled::Db;
pub struct AppStore {
    db: Db,
    updated_apps: Vec<(String, String)>,
}
impl AppStore {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            updated_apps: vec![],
        }
    }
    pub fn handle_request(req: AppStoreRequest) -> AppStoreResponse {
        match req {
            AppStoreRequest::AppExists { owner, app_name } => todo!(),
            AppStoreRequest::UpsertApp {
                owner,
                app_name,
                code,
            } => todo!(),
            AppStoreRequest::RemoveApp { owner, app_name } => todo!(),
            AppStoreRequest::DisableApp { owner, app_name } => todo!(),
            AppStoreRequest::EnableApp { owner, app_name } => todo!(),
            AppStoreRequest::RenameApp {
                owner,
                old_name,
                new_name,
            } => todo!(),
            AppStoreRequest::GetAppCode { owner, app_name } => todo!(),
            AppStoreRequest::GetOwnedApps { owner } => todo!(),
            AppStoreRequest::RequestUpdates => todo!(),
        }
    }
}
