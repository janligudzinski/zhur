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
        tree.clear().unwrap();
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
        let (exists, enabled) = self.app_exists(owner, app_name);
        let app_data = ApplicationData {
            owner: owner.to_string(),
            app_name: app_name.to_string(),
            enabled: if !exists { true } else { enabled },
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
    fn rename_app(&self, owner: &str, old_name: &str, new_name: &str) -> bool {
        if self.app_exists(owner, new_name).0 {
            return false;
        }
        let tree = self.db.open_tree(owner).unwrap();
        let mut old_app: ApplicationData = tree
            .get(old_name)
            .unwrap()
            .map(|bytes| deserialize(&bytes).unwrap())
            .unwrap();
        old_app.app_name = new_name.to_string();
        let old_app_code: Vec<u8> = tree
            .get(old_name.to_string() + "_code")
            .unwrap()
            .unwrap()
            .to_vec();
        tree.insert(new_name, serialize(&old_app).unwrap()).unwrap();
        tree.insert(new_name.to_string() + "_code", old_app_code)
            .unwrap();
        tree.remove(old_name).unwrap();
        tree.remove(old_name.to_string() + "_code").unwrap();
        true
    }
    fn get_owned_apps(&self, owner: &str) -> Vec<ApplicationData> {
        let tree = self.db.open_tree(owner).unwrap();
        tree.iter()
            .filter_map(|x| x.ok())
            .map(|(k, v)| {
                let k = std::str::from_utf8(&k).unwrap().to_string();
                (k, v)
            })
            .filter(|(k, _v)| !k.ends_with("_code"))
            .map(|(_k, v)| deserialize::<ApplicationData>(&v).unwrap())
            .collect()
    }
    fn get_app_code(&self, owner: &str, app_name: &str) -> Option<Vec<u8>> {
        let tree = self.db.open_tree(owner).unwrap();
        tree.get(app_name.to_string() + "_code")
            .unwrap()
            .map(|i| i.to_vec())
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
            } => {
                let success = self.rename_app(&owner, &old_name, &new_name);
                AppStoreResponse::AppRenamed(success)
            }
            AppStoreRequest::GetAppCode { owner, app_name } => {
                let code = self.get_app_code(&owner, &app_name).unwrap();
                AppStoreResponse::Code { code }
            }
            AppStoreRequest::GetOwnedApps { owner } => {
                let apps = self.get_owned_apps(&owner);
                AppStoreResponse::Apps { apps }
            }
            AppStoreRequest::RequestUpdates => {
                let apps = self.get_updates();
                self.clear_updates();
                AppStoreResponse::AppsChanged { apps }
            }
        }
    }
}
