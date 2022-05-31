use crate::data;
use crate::todo::*;
use serde::de::DeserializeOwned;
use zhur_sdk::http::*;
use zhur_sdk::web::*;
const INDEX_HTML: &str = include_str!("../res/index.html");
const INDEX_JS: &[u8] = include_bytes!("../res/index.js");

pub fn index() -> Html {
    Html(INDEX_HTML.into())
}
pub fn index_js() -> File {
    File(INDEX_JS.to_vec(), Some(String::from("text/javascript")))
}

/// Helper function for extracting JSON data from request bodies.
fn json_from_body<T: DeserializeOwned>(req: &HttpReq) -> Result<T, impl Responder> {
    let body = match &req.body {
        None => return Err(StatusCode(422, Text(String::from("No body was provided.")))),
        Some(body) => body,
    };
    let deser_result = match body {
        HttpBody::Binary(bytes) => serde_json::from_slice::<T>(bytes),
        HttpBody::Text(text) => serde_json::from_str::<T>(text),
    };
    deser_result.map_err(|e| {
        StatusCode(
            422,
            Text(format!(
                "There was an error extracting data out of the request body: {}",
                e.to_string()
            )),
        )
    })
}

pub fn get_todos() -> impl Responder {
    let todos: Vec<Todo> = crate::data::get_all_todos();
    Json(todos)
}
pub fn add_todo(req: &HttpReq) -> Result<impl Responder, impl Responder> {
    let new_todo: TodoNewRequest = match json_from_body(req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    data::add_todo(new_todo.text);
    Ok(())
}
pub fn mark_todo(req: &HttpReq) -> Result<(), impl Responder> {
    let marked: TodoMarkRequest = match json_from_body(req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    data::mark_todo(marked.id, marked.complete);
    Ok(())
}
pub fn edit_todo(req: &HttpReq) -> Result<(), impl Responder> {
    let edit: TodoEditRequest = match json_from_body(req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    data::edit_todo(edit.id, edit.text);
    Ok(())
}
pub fn clear_complete_todos() -> impl Responder {
    data::clear_done_todos();
    ()
}
pub fn delete_todo(req: &HttpReq) -> Result<(), impl Responder> {
    let del: TodoDelRequest = match json_from_body(req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    data::delete_todo(del.id);
    Ok(())
}
