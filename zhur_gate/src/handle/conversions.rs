use http::{HeaderValue, header::HeaderName};
use hyper::{Body, Response};
use super::{FullRequest, HttpReq, InvocationError};
use std::collections::BTreeMap;
use zhur_common::log::*;
use zhur_invk::{HttpRes, Invocation};
/// Simplifies a `FullRequest` into an `HttpReq` of ours.
pub async fn simplify_req(req: FullRequest) -> Result<HttpReq, InvocationError> {
    use http::header::COOKIE;
    let method = req.req.method().to_string();
    let uri = req.req.uri();
    let path = uri.path().to_owned();
    let query_params = match uri.query() {
        Some(s) => parse_query_string(s),
        None => BTreeMap::new(),
    };
    // dbg!(&query_params);
    let cookie_str = match req.req.headers().get(COOKIE) {
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
    let mut headers = BTreeMap::new();
    for (name, val) in req.req.headers().iter() {
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
    let body = match hyper::body::to_bytes(req.req.into_body()).await {
        Ok(b) => b.to_vec(),
        Err(_) => return Err(InvocationError::MalformedRequest),
    };
    Ok(HttpReq {
        body,
        path,
        method,
        cookies,
        headers,
        query_params,
        ip_addr: req.ip.clone(),
    })
}

impl FullRequest {
    /// Turns a `FullRequest` into an `Invocation`.
    pub async fn into_invoc(self) -> Result<Invocation, InvocationError> {
        use http::header::HOST;
        use zhur_common::bincode::serialize;
        let host = match self.req.headers().get(HOST) {
            Some(s) => match s.to_str() {
                Ok(s) => s,
                Err(_) => {
                    warn!("Received an HTTP request with a non-UTF-text Host header, returning malformed ID error.");
                    return Err(InvocationError::MalformedId("(not valid UTF-8 text)".into()))
                }
            },
            None => {
                warn!("Received an HTTP request with no Host header, returning a no ID error.");
                return Err(InvocationError::NoId)
            }
        }.to_owned();
        let segments = host.split('.').collect::<Vec<_>>();
        if segments.is_empty() {
            warn!("Received an HTTP request with an empty Host header, returning a no ID error.");
            return Err(InvocationError::NoId);
        } else if segments.len() < 2 {
            warn!("Received an HTTP request with a Host header that could not be transformed into an app ID: \"{}\"", host);
            return Err(InvocationError::MalformedId(host.into()));
        }
        let req_simple = simplify_req(self).await?;
        let req_bytes = match serialize(&req_simple) {
            Ok(b) => b,
            Err(_) => return Err(InvocationError::MalformedRequest),
        };
        let result = Invocation {
            owner: segments[1].to_owned(),
            app_name: segments[0].to_owned(),
            payload: req_bytes,
        };
        Ok(result)
    }
}
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

/// Builds a hyper `Response` from an `HttpRes`.
pub fn realize_response(res: HttpRes) -> Response<Body> {
    let mut builder = Response::builder();
    {
        let headers = builder.headers_mut().unwrap();
        match res.set_cookie {
            Some((key, val)) => {
                headers.insert(
                    "Set-Cookie", HeaderValue::from_str(&set_cookie(&key, &val)).unwrap()
                );
            },
            None => ()
        };
        for (key, val) in &res.headers {
            let h_name = HeaderName::from_bytes(key.as_bytes());
            let h_val = HeaderValue::from_bytes(val.as_bytes());
            match (h_name, h_val) {
                (Ok(n), Ok(v)) => {
                    headers.insert(n, v);
                },
                _ => ()
            }
        }
    }
    builder
    .status(res.status)
    .body(res.body.into())
    .unwrap()

}
/// Generates a Set-Cookie header.
fn set_cookie(key: &str, val: &str) -> String {
    let mut output = String::from("Set-Cookie: ");
    output.push_str(key);
    output.push('=');
    output.push_str(val);
    output
}