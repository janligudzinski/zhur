use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod conversions;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// An enum to represent possible HTTP bodies.
pub enum HttpBody {
    /// Plain text or a format based on it.
    Text(String),
    /// Binary data.
    Binary(Vec<u8>),
}
impl Default for HttpBody {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A Serde-friendly representation of a HTTP request.
pub struct HttpReq {
    /// Non-body parts of the request.
    pub parts: HttpReqParts,
    /// The body of the request.
    pub body: HttpBody,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A Serde-friendly representation of a HTTP request's parts.
pub struct HttpReqParts {
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A Serde-friendly representation of an HTTP response.
pub struct HttpRes {
    /// Headers represented as pairs of `String`s.
    pub headers: BTreeMap<String, String>,
    /// The body of the response.
    pub body: HttpBody,
    /// The status code.
    pub status: u16,
}
impl Default for HttpRes {
    fn default() -> Self {
        Self {
            headers: BTreeMap::new(),
            body: HttpBody::default(),
            status: 200,
        }
    }
}
