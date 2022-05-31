use crate::routes;
use zhur_sdk::http::*;
use zhur_sdk::web::*;

fn not_found() -> impl Responder {
    StatusCode(404, Text(String::from("No such route.")))
}

fn unsupported_method() -> impl Responder {
    let html = "<html><body><h1>405 Method Not Allowed</h1><p>The server does not allow making this request with the method given.</p></body></html>".to_string();
    StatusCode(405, Html(html))
}

pub fn route(req: &HttpReq, res: &mut HttpRes) {
    res.headers
        .insert("Access-Control-Allow-Origin".into(), "*".into()); // for localhost development
    match req.parts.path.as_str() {
        "/" => routes::index().modify_response(res),
        "/index.js" => routes::index_js().modify_response(res),
        "/todos" => match req.parts.method.as_str() {
            "GET" => routes::get_todos().modify_response(res),
            _ => unsupported_method().modify_response(res),
        },
        "/mark" => match req.parts.method.as_str() {
            "POST" => routes::mark_todo(req).modify_response(res),
            _ => unsupported_method().modify_response(res),
        },
        "/edit" => match req.parts.method.as_str() {
            "POST" => routes::edit_todo(req).modify_response(res),
            _ => unsupported_method().modify_response(res),
        },
        "/clear_complete" => match req.parts.method.as_str() {
            "POST" => routes::clear_complete_todos().modify_response(res),
            _ => unsupported_method().modify_response(res),
        },
        "/del" => match req.parts.method.as_str() {
            "POST" => routes::delete_todo(req).modify_response(res),
            _ => unsupported_method().modify_response(res),
        },
        "/add" => match req.parts.method.as_str() {
            "POST" => routes::add_todo(req).modify_response(res),
            _ => unsupported_method().modify_response(res),
        },
        _ => not_found().modify_response(res),
    }
}
