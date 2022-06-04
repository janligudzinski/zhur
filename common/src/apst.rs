use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ApplicationData {
    pub owner: String,
    pub app_name: String,
    pub enabled: bool,
}

#[derive(Deserialize, Serialize)]
pub enum AppStoreRequest {
    /// Check if an app exists / can be launched.
    AppExists { owner: String, app_name: String },
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
    /// Rename an app.
    RenameApp {
        owner: String,
        old_name: String,
        new_name: String,
    },
    /// Get the code for an app.
    GetAppCode { owner: String, app_name: String },
    /// Get the metadata for all apps for a given user.
    GetOwnedApps { owner: String },
    /// Request state updates since last request made. This message type was implemented mostly so as not to have to implement pub/sub updates.
    RequestUpdates,
}

#[derive(Deserialize, Serialize)]
pub enum AppStoreResponse {
    AppExistence(bool),
    AppUpserted,
    AppRemoved,
    AppRenamed(bool),
    AppDisabled,
    AppEnabled,
    Code { code: Vec<u8> },
    AppsChanged { apps: Vec<(String, String)> },
}
