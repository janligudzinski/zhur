fn hello(name: String) -> String {
    if name.contains("panic") {
        panic!("The special panic name was invoked.");
    }
    let hello_text = format!(
        "Hello, {}, this is a WASM app speaking, through a convenience macro too!",
        name
    );
    hello_text
}
zhur_sdk::text_function!(hello);
