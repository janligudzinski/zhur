use askama::Template;
use zhur_sdk::http::*;
use zhur_sdk::http_function;
#[derive(Template)]
#[template(path = "index.html")]
/// This repetitive struct, as well as its `From<HttpReq>` impl, are required because Askama can't deal with BTreeMaps well.
struct ReqTemplate {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    cookies: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    body: ReqBody,
    owner: String,
    app_name: String,
    timestamp: i64,
}
enum ReqBody {
    ValidText(String),
    Bytes(Vec<u8>),
    Empty,
}
impl From<HttpReq> for ReqTemplate {
    fn from(req: HttpReq) -> Self {
        let now = zhur_sdk::datetime::now();
        let body = match req.body {
            Some(HttpBody::Binary(bytes)) => ReqBody::Bytes(bytes),
            Some(HttpBody::Text(text)) => ReqBody::ValidText(text),
            None => ReqBody::Empty,
        };
        Self {
            method: req.parts.method,
            path: req.parts.path,
            headers: req.parts.headers.into_iter().collect(),
            cookies: req.parts.cookies.into_iter().collect(),
            query_params: req.parts.query_params.into_iter().collect(),
            body,
            owner: String::new(),
            app_name: String::new(),
            timestamp: now.timestamp(),
        }
    }
}

fn echo(req: &HttpReq, res: &mut HttpRes) {
    let mut tmpl = ReqTemplate::from(req.clone());
    let whoami = zhur_sdk::meta::whoami();
    tmpl.owner = whoami.0;
    tmpl.app_name = whoami.1;
    let responder = Html(tmpl.render().unwrap_or("Rendering error".to_string()));
    responder.modify_response(res);
}
http_function!(echo);
