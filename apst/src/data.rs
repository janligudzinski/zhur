use common::{
    apst::{AppStoreRequest, AppStoreResponse, ApplicationData},
    prelude::bincode::deserialize,
};
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
    fn app_exists(&self, owner: &str, app_name: &str) -> bool {
        let tree = self.db.open_tree(owner).unwrap();
        let app_data = self.db.get(app_name).unwrap();
        match app_data {
            None => false,
            Some(bytes) => {
                let data: ApplicationData = deserialize(&bytes).unwrap();
                data.enabled
            }
        }
    }
    pub fn handle_request(&self, req: AppStoreRequest) -> AppStoreResponse {
        match req {
            AppStoreRequest::AppExists { owner, app_name } => {
                AppStoreResponse::AppExistence(self.app_exists(&owner, &app_name))
            }
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
