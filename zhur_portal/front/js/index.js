const m = require("mithril");
let main = document.querySelector("main");
const HelloComponent = {
    view: () => {
        return m("p", "Hello world, Mithril here!");
    }
}
m.mount(main, HelloComponent);