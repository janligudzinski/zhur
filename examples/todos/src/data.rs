use serde::{Deserialize, Serialize};
use zhur_sdk::svc::kv::*;
use crate::todo::Todo;

#[derive(Deserialize, Serialize)]
struct Todos {
    todos: Vec<Todo>,
    counter: i32,
}
impl Todos {
    fn load() -> Self {
        match kv_get("todo", "todos") {
            Some(s) => s,
            None => Self {
                todos: Vec::new(),
                counter: 0,
            }
        }
    }
    fn save(&self) {
       kv_set("todo", "todos", &self);
    }
    fn add(&mut self, text: String) {
        self.counter += 1;
        self.todos.push(Todo {complete: false, text, id: self.counter});
    }
}
pub fn get_all_todos() -> Vec<Todo> {
    let todos = Todos::load();
    todos.todos
}

pub fn mark_todo(id: i32, done: bool) {
    let mut todos = Todos::load();
    for each in todos.todos.iter_mut() {
        if each.id == id {
            each.complete = done;
        }
    }
    todos.save();
}
pub fn edit_todo(id: i32, text: String) {
    let mut todos = Todos::load();
    for each in todos.todos.iter_mut() {
        if each.id == id {
            each.text = text.clone();
        }
    }
    todos.save();
}
pub fn delete_todo(id: i32) {
    let mut todos = Todos::load();
    todos.todos.retain(|each| each.id != id);
    todos.save();
}
pub fn clear_done_todos() {
    let mut todos = Todos::load();
    todos.todos.retain(|each| !each.complete);
    todos.save();
}
pub fn add_todo(text: String) {
    let mut todos = Todos::load();
    todos.add(text);
    todos.save();
}