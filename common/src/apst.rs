use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum AppStoreRequest {
    /// Add or update an app.
    UpsertApp {
        owner: String,
        app_name: String,
        code: Vec<u8>,
    },
    /// Remove an app.
    RemoveApp { owner: String, app_name: String },
    /// Disable an app temporarily.
    DisableApp { owner: String, app_name: String },
    /// Enable a previously disabled app.
    EnableApp { owner: String, app_name: String },
    /// Get the code for an app.
    GetAppCode { owner: String, app_name: String },
    /// Get the metadata for all apps for a given user.
    GetOwnedApps { owner: String },
}
