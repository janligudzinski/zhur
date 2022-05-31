use http::{request::Parts, Request};
use log::*;
use std::collections::BTreeMap;

use super::{HttpBody, HttpReq, HttpReqParts};

impl From<Parts> for HttpReqParts {
    fn from(p: Parts) -> Self {
        // Extract the simple stuff.
        let method = p.method.to_string();
        let uri = p.uri;
        let path = uri.path().to_owned();

        // Extract query parameters.
        let query_params = match uri.query() {
            Some(s) => parse_query_string(s),
            None => BTreeMap::new(),
        };

        // Extract cookies.
        let cookie_str = match p.headers.get(http::header::COOKIE) {
            Some(hv) => match hv.to_str() {
                Ok(s) => Some(s),
                Err(_) => {
                    warn!("Got a Cookie string that is not a valid ASCII string. Discarding.");
                    None
                }
            },
            None => None,
        };
        let cookies = match cookie_str {
            Some(s) => cookie_map(s),
            None => BTreeMap::new(),
        };

        // Extract other headers.

        let mut headers = BTreeMap::new();
        for (name, val) in p.headers.iter() {
            if name == "Cookie" {
                continue;
            }
            match val.to_str() {
                Ok(s) => {
                    headers.insert(name.to_string(), s.to_owned());
                }
                Err(_) => {
                    warn!("Found a header that could not be parsed as a string. Discarding.");
                    continue;
                }
            }
        }

        HttpReqParts {
            cookies,
            headers,
            method,
            path,
            query_params,
        }
    }
}

impl From<Request<Vec<u8>>> for HttpReq {
    fn from(value: Request<Vec<u8>>) -> Self {
        let (parts, body) = value.into_parts();
        HttpReq {
            parts: parts.into(),
            body: HttpBody::Binary(body),
        }
    }
}
impl From<Request<String>> for HttpReq {
    fn from(value: Request<String>) -> Self {
        let (parts, body) = value.into_parts();
        HttpReq {
            parts: parts.into(),
            body: HttpBody::Text(body),
        }
    }
}

// Helper functions below.

/// Parses a query string of the form "a=b&c=d" into params. Note: Hyper takes care of extracting said string out of a URI already, so no need to worry about ?.
fn parse_query_string(s: &str) -> BTreeMap<String, String> {
    let mut output = BTreeMap::new();
    let pairs = s.split("&");
    for pair in pairs {
        let mut param_val = pair.split("=");
        let param = match param_val.next() {
            Some(p) => p,
            None => continue,
        };
        let val = match param_val.next() {
            Some(v) => v,
            None => continue,
        };
        output.insert(param.to_owned(), val.to_owned());
    }
    output
}

/// Produces a map of cookies to their values given a cookie string.
fn cookie_map(cookie_str: &str) -> BTreeMap<String, String> {
    let mut output = BTreeMap::new();
    for cookie in cookie_str.split("; ") {
        let mut name_val = cookie.split("=");
        let name = match name_val.next() {
            Some(n) => n,
            None => continue,
        };
        let val = match name_val.next() {
            Some(v) => v,
            None => continue,
        };
        output.insert(name.to_owned(), val.to_owned());
    }
    output
}
