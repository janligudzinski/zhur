use zhur_sdk::web::*;
use zhur_sdk::http::*;
use crate::todo::*;
use crate::data;
const INDEX_HTML: &str = include_str!("../res/index.html");
const INDEX_JS: &[u8] = include_bytes!("../res/index.js");

pub fn index() -> Html {
    Html(INDEX_HTML.into())
}
pub fn index_js() -> File {
    File(INDEX_JS.to_vec(), Some(String::from("text/javascript")))
}

pub fn get_todos() -> Result<Json<Vec<Todo>>, StatusCode<Html>> {
    let todos: Vec<Todo> = crate::data::get_all_todos();
    Ok(Json(todos))
}
pub fn add_todo(req: &HttpReq) -> Result<(), StatusCode<Text>> {
    let new_todo: TodoNewRequest = match serde_json::from_slice(&req.body) {
        Ok(todo) => todo,
        Err(_) => {
            return Err(StatusCode(
                400, 
                Text(String::from("Your request body couldn't be deserialized into a new Todo."))
                )
            )
        }
    };
    data::add_todo(new_todo.text);
    Ok(())
}
pub fn mark_todo(req: &HttpReq) -> Result<(), StatusCode<Text>> {
    let marked: TodoMarkRequest = match serde_json::from_slice(&req.body) {
        Ok(todo) => todo,
        Err(_) => {
            return Err(StatusCode(
                400, 
                Text(String::from("Your request body couldn't be deserialized into a Todo mark request."))
                )
            )
        }
    };
    data::mark_todo(marked.id, marked.complete);
    Ok(())
}
pub fn edit_todo(req: &HttpReq) -> Result<(), StatusCode<Text>> {
    let edit: TodoEditRequest = match serde_json::from_slice(&req.body) {
        Ok(todo) => todo,
        Err(_) => {
            return Err(StatusCode(
                400, 
                Text(String::from("Your request body couldn't be deserialized into a Todo edit request."))
                )
            )
        }
    };
    data::edit_todo(edit.id, edit.text);
    Ok(())
}
pub fn clear_complete_todos() -> Result<(), StatusCode<Text>> {
    data::clear_done_todos();
    Ok(())
}
pub fn delete_todo(req: &HttpReq) -> Result<(), StatusCode<Text>> {
    let del: TodoDelRequest = match serde_json::from_slice(&req.body) {
        Ok(todo) => todo,
        Err(_) => return Err(StatusCode(400, Text("Couldn't parse your delete todo request.".to_string())))
    };
    data::delete_todo(del.id);
    Ok(())
}