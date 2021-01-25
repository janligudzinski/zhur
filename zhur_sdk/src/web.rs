use serde::Serialize;
use super::http::*;
use mime_sniffer::MimeTypeSniffer;

/// This is a trait for easily producing `HttpRes`es. *Heavily* inspired by Rocket's trait of the same name.
///
/// `Responder` comes with implementations for some common types of data - plain text, HTML, JSON/JSON:API, arbitrary binary files - as well as a status code modifier.
/// You can, of course, implement `Responder` for your own types to suit your needs.
pub trait Responder {
    /// Given a mutable reference to a response, fills it with the relevant data. Consumes itself.
    fn modify_response(self, res: &mut HttpRes);
}
/// Modifies the status code of the response produced by another `Responder`.
pub struct StatusCode<R: Responder>(pub u16, pub R);
impl<R: Responder> Responder for StatusCode<R> {
    fn modify_response(self, res: &mut HttpRes) {
        self.1.modify_response(res);
        res.status = self.0;
    }
}
/// Plain UTF-8 text response.
pub struct Text(pub String);
impl Responder for Text {
    fn modify_response(self, res: &mut HttpRes) {
        res.body = self.0.into_bytes();
        res.headers.insert("Content-Type".to_owned(), "text/plain; charset=utf-8".to_owned());
    }
}
/// HTML response.
pub struct Html(pub String);
impl Responder for Html {
    fn modify_response(self, res: &mut HttpRes) {
        res.body = self.0.into_bytes();
        res.headers.insert("Content-Type".to_owned(), "text/html; charset=utf-8".to_owned());
    }
}
/// JSON response.
pub struct Json<S: Serialize>(pub S);
impl<S: Serialize> Responder for Json<S> {
    fn modify_response(self, res: &mut HttpRes) {
        res.body = serde_json::to_vec(&self.0).unwrap();
        res.headers.insert("Content-Type".to_owned(), "application/json".to_owned());
    }
}
/// JSON:API response.
pub struct JsonApi<S: Serialize>(pub S);
impl<S: Serialize> Responder for JsonApi<S> {
    fn modify_response(self, res: &mut HttpRes) {
        res.body = serde_json::to_vec(&self.0).unwrap();
        res.headers.insert("Content-Type".to_owned(), "application/vnd.api+json".to_owned());
    }
}
/// Arbitrary file response. If a custom MIME type is not provided, `application/octet-stream` is assumed.
/// The MIME type can be sniffed with the convenience method `new`.
pub struct File(pub Vec<u8>, pub Option<String>);
impl Responder for File {
    fn modify_response(self, res: &mut HttpRes) {
        res.body = self.0;
        res.headers.insert("Content-Type".to_owned(), self.1.unwrap_or("application/octet-stream".to_owned()));
    }
}
impl File {
    /// Constructs a `File` and automatically sniffs its MIME type. If none can be established, `application/octet-stream` is assumed.
    pub fn new(bytes: Vec<u8>) -> Self {
        let mime = match bytes.sniff_mime_type() {
            Some(mime) => {mime.to_owned()}
            None => {"application/octet-stream".to_owned()}
        };
        Self(bytes, Some(mime))
    }
}
/// Default implementation for things that may or may not be there.
impl<R: Responder> Responder for Option<R> {
    fn modify_response(self, res: &mut HttpRes) {
        match self {
            Some(r) => r.modify_response(res),
            None => {
                res.status = 404;
                Text("Default 404 Not Found handler".into()).modify_response(res);
            }
        }
    }
}
/// Any `Result` is a `Responder` if both its types are themselves `Responder`s.
impl<R: Responder, E: Responder> Responder for Result<R, E> {
    fn modify_response(self, res: &mut HttpRes) {
        match self {
            Ok(r) => r.modify_response(res),
            Err(e) => e.modify_response(res)
        }
    }
}
/// Empty response implementation.
impl Responder for () {
    fn modify_response(self, _res: &mut HttpRes) {
    }
}