use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Deserialize, Serialize)]
/// A Serde-friendly representation of a HTTP request.
pub struct HttpReq {
    /// The method of the request.
    pub method: String,
    /// The path of the request.
    pub path: String,
    /// Headers represented as pairs of `String`s.
    pub headers: BTreeMap<String, String>,
    /// Query strings represented as pairs of `String`s.
    pub query_params: BTreeMap<String, String>,
    /// Cookies represented as pairs of `String`s.
    pub cookies: BTreeMap<String, String>,
    /// The IP address of the requester.
    pub ip_addr: String,
    /// The body of the request, as bytes.
    pub body: Vec<u8>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// A Serde-friendly representation of an HTTP response.
pub struct HttpRes {
    /// Headers represented as pairs of `String`s.
    pub headers: BTreeMap<String, String>,
    /// An optional `Set-Cookie` header. Setting multiple cookies is not supported and additional attributes aren't supported yet either.
    pub set_cookie: Option<(String, String)>,
    /// The body of the response, as bytes.
    pub body: Vec<u8>,
    /// The status code.
    pub status: u16,
}
impl Default for HttpRes {
    fn default() -> Self {
        Self {
            headers: BTreeMap::new(),
            set_cookie: None,
            body: Vec::new(), // compiler throws a fit on vec![]
            status: 200,
        }
    }
}
