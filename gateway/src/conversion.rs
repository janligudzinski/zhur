use axum::{http::Response, response::IntoResponse};
use shared::http::{HttpBody, HttpRes};

pub struct HttpResWrapper(pub HttpRes);
impl IntoResponse for HttpResWrapper {
    fn into_response(self) -> axum::response::Response {
        let mut response = Response::builder();
        response = response.status(self.0.status);
        for (key, val) in self.0.headers {
            response = response.header(key, val);
        }
        match self.0.body {
            HttpBody::Binary(bytes) => response
                .header("Content-Type", "application/octet-stream")
                .body(axum::body::boxed(axum::body::Body::from(bytes)))
                .unwrap(),
            HttpBody::Text(text) => response
                .body(axum::body::boxed(axum::body::Body::from(text)))
                .unwrap(),
        }
    }
}
