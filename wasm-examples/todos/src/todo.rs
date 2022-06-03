use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub complete: bool,   
}
#[derive(Deserialize)]
pub struct TodoMarkRequest {
    pub id: i32,
    pub complete: bool,
}

#[derive(Deserialize)]
pub struct TodoEditRequest {
    pub id: i32,
    pub text: String
}

#[derive(Deserialize)]
pub struct TodoDelRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct TodoNewRequest {
    pub text: String,
}