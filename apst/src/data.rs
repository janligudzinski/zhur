use common::{
    apst::{AppStoreRequest, AppStoreResponse, ApplicationData},
    prelude::bincode::{deserialize, serialize},
};
use sled::Db;
pub struct AppStore {
    db: Db,
}
impl AppStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
    fn clear_updates(&self) {
        let tree = self.db.open_tree("updates").unwrap();
        tree.clear();
    }
    fn get_updates(&self) -> Vec<(String, String)> {
        let tree = self.db.open_tree("updates").unwrap();
        let mut vals = vec![];
        for (_key, bytes) in tree.iter().filter_map(|r| r.ok()) {
            let owner_name = deserialize::<(String, String)>(&bytes).unwrap();
            vals.push(owner_name);
        }
        vals
    }
    fn register_update(&self, owner: &str, app_name: &str) {
        let tree = self.db.open_tree("updates").unwrap();
        tree.insert(
            self.db.generate_id().unwrap().to_le_bytes(),
            serialize(&(owner, app_name)).unwrap(),
        )
        .unwrap();
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
    fn remove_app(&self, owner: &str, app_name: &str) {
        let tree = self.db.open_tree(owner).unwrap();
        let code_key = app_name.to_string() + "_code";
        tree.remove(app_name).unwrap();
        tree.remove(code_key).unwrap();
    }
    fn set_app_state(&self, owner: &str, app_name: &str, enabled: bool) {
        let tree = self.db.open_tree(owner).unwrap();
        let app_data = tree.get(app_name).unwrap();
        match app_data {
            None => panic!("Tried to disable or enable a nonexistent app!"),
            Some(bytes) => {
                let mut data: ApplicationData = deserialize(&bytes).unwrap();
                data.enabled = enabled;
                tree.insert(app_name, serialize(&data).unwrap()).unwrap();
            }
        }
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
                self.register_update(&owner, &app_name);
                AppStoreResponse::AppUpserted
            }
            AppStoreRequest::RemoveApp { owner, app_name } => {
                self.remove_app(&owner, &app_name);
                AppStoreResponse::AppRemoved
            }
            AppStoreRequest::DisableApp { owner, app_name } => {
                self.set_app_state(&owner, &app_name, false);
                AppStoreResponse::AppDisabled
            }
            AppStoreRequest::EnableApp { owner, app_name } => {
                self.set_app_state(&owner, &app_name, true);
                AppStoreResponse::AppEnabled
            }
            AppStoreRequest::RenameApp {
                owner,
                old_name,
                new_name,
            } => todo!(),
            AppStoreRequest::GetAppCode { owner, app_name } => todo!(),
            AppStoreRequest::GetOwnedApps { owner } => todo!(),
            AppStoreRequest::RequestUpdates => {
                let apps = self.get_updates();
                self.clear_updates();
                AppStoreResponse::AppsChanged { apps }
            }
        }
    }
}
