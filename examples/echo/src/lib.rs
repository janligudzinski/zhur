use zhur_sdk::{handle_http, web::*};
use zhur_sdk::http::*;
use askama::Template;
#[derive(Template)]
#[template(path = "index.html")]
/// This repetitive struct, as well as its `From<HttpReq>` impl, are required because Askama can't deal with BTreeMaps well.
struct ReqTemplate{
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    cookies: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    ip_addr: String,
    body: ReqBody,
    owner: String,
    app_name: String,
    timestamp: i64,
}
enum ReqBody {
    ValidText(String),
    Bytes(Vec<u8>),
    Empty
}
impl From<HttpReq> for ReqTemplate {
    fn from(req: HttpReq) -> Self {
        let now = zhur_sdk::svc::datetime::now();
        Self {
            method: req.method, path: req.path, ip_addr: req.ip_addr,
            headers: req.headers.into_iter().collect(),
            cookies: req.cookies.into_iter().collect(),
            query_params: req.query_params.into_iter().collect(),
            body: {
                if req.body.is_empty() {
                    ReqBody::Empty
                } else {
                    match std::str::from_utf8(&req.body) {
                        Ok(s) => ReqBody::ValidText(s.to_string()),
                        _ => ReqBody::Bytes(req.body)
                    }
                }
            },
            owner: String::new(),
            app_name: String::new(),
            timestamp: now.timestamp()
        }
    }
}

fn echo(req: &HttpReq, res: &mut HttpRes) {
    let mut tmpl = ReqTemplate::from(req.clone());
    let whoami = zhur_sdk::svc::meta::whoami();
    tmpl.owner = whoami.0;
    tmpl.app_name = whoami.1;
    let responder = Html(tmpl.render().unwrap_or("Rendering error".to_string()));
    responder.modify_response(res);
}
handle_http!(echo);