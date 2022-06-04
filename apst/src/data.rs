use common::{
    apst::{AppStoreRequest, AppStoreResponse, ApplicationData},
    prelude::bincode::{deserialize, serialize},
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
    fn app_exists(&self, owner: &str, app_name: &str) -> (bool, bool) {
        let tree = self.db.open_tree(owner).unwrap();
        let app_data = tree.get(app_name).unwrap();
        match app_data {
            None => (false, false),
            Some(bytes) => {
                let data: ApplicationData = deserialize(&bytes).unwrap();
                (true, data.enabled)
            }
        }
    }
    fn upsert_app(&self, owner: &str, app_name: &str, code: Vec<u8>) {
        let tree = self.db.open_tree(owner).unwrap();
        let app_data = ApplicationData {
            owner: owner.to_string(),
            app_name: app_name.to_string(),
            enabled: self.app_exists(owner, app_name).1,
        };
        tree.insert(app_name, serialize(&app_data).unwrap())
            .unwrap();
        tree.insert(app_name.to_string() + "_code", code).unwrap();
    }
    pub fn handle_request(&self, req: AppStoreRequest) -> AppStoreResponse {
        match req {
            AppStoreRequest::AppExists { owner, app_name } => {
                AppStoreResponse::AppExistence(self.app_exists(&owner, &app_name).1)
            }
            AppStoreRequest::UpsertApp {
                owner,
                app_name,
                code,
            } => {
                self.upsert_app(&owner, &app_name, code);
                AppStoreResponse::AppUpserted
            }
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
