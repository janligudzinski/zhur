use common::prelude::tokio::net::UnixStream;
use ipc::UnixClient;
pub struct AppRepo {
    path: String,
}
impl AppRepo {
    pub fn new(path: &str) -> Self {
        Self { path: path.into() }
    }

    pub async fn get_connection(&self) -> UnixClient {
        let stream = UnixStream::connect(&self.path).await.unwrap();
        let client = UnixClient::new(1024 * 1024 * 20, stream);
        client
    }
}
