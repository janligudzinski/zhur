const root = document.getElementById("main");

let state = {
    add_todo_text: "",
    todos: []
};

const edit_todo = (id, text) => {
    fetch("/edit", {
        method: "POST",
        body: JSON.stringify({id: id, text: text})
    })
    .then(response => {
        if (response.ok) {
            load_todos();
        } else {
            alert("Error");
        }
    });
}

const post_todo = (text) => {
    fetch("/add", {
        method: "POST",
        body: JSON.stringify({text: text})
    })
    .then(response => {
        if (response.ok) {
            state.add_todo_text = "";
            load_todos();
        } else {
            alert("Error");
        }
    });
}
const load_todos = () => {
    fetch("/todos")
    .then(response => response.json())
    .then(data => {
        state.todos = data;
        m.redraw();
    })
}
const mark_todo = (id, complete) => {
    fetch("/mark", {
        method: "POST",
        body: JSON.stringify({id: id, complete: complete})
    })
    .then(response => {
        if (response.ok) {
            load_todos();
        } else {
            alert("Error");
        }
    });
}
const del_todo = (id) => {
    fetch("/del", {
        method: "POST",
        body: JSON.stringify({id: id})
    })
    .then(response => {
        if (response.ok) {
            load_todos();
        } else {
            alert("Error");
        }
    });
}
const clear_complete = () => {
    fetch("/clear_complete", {
        method: "POST"
    })
    .then(response => {
        if (response.ok) {
            load_todos();
        } else {
            alert("Error");
        }
    });
}

const TodoRow = {
    view: (vnode) => {
        let edit_button = m("button", {
            onclick: () => {
                let new_text = window.prompt("Enter a new description for the todo item:", vnode.attrs.text);
                edit_todo(vnode.attrs.id, new_text);
            }
        }, "Edit");
        let mark_button = m("button", {
            onclick: () => mark_todo(vnode.attrs.id, !vnode.attrs.complete)
        }, "Toggle");
        let del_button = m("button", {
            onclick: () => del_todo(vnode.attrs.id)
        }, "Delete");
        return m("p", {class: vnode.attrs.complete ? "complete" : null}, [edit_button, mark_button, del_button, `Todo #${vnode.attrs.id}: ${vnode.attrs.text}`]);
    }
}

const TodoAdder = {
    view: () => {
        let input = m("input", {type: "text", value: state.add_todo_text, oninput: (e) => {
            state.add_todo_text = e.target.value
        }})
        let add_button = m("button", {disabled: state.add_todo_text.length < 1, onclick: () => post_todo(state.add_todo_text)}, "Add todo");
        return m("div", [input, add_button]);
    }
}

const TodoList = {
    view: () => {
        let content = [m("h2", "Todos")];
        if (state.todos.length < 1) {
            content.push(m("p", "No todos yet. Add some!"));
        } else {
            state.todos.forEach(each => {
                content.push(m(TodoRow, {id: each.id, text: each.text, complete: each.complete}))
            });
        }
        return m.fragment(content);
    }
};

const MainView = {
    view: () => {
        return m("div", [
            m(TodoList),
            m(TodoAdder),
            m("button", {onclick: () => clear_complete()}, "Clear completed todos")
        ]);
    }
};

m.mount(root, MainView);
load_todos();