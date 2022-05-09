use wapc_guest as wapc;

#[no_mangle]
pub fn wapc_init() {
    wapc::register_function("hello", hello)
}

fn hello(msg: &[u8]) -> wapc::CallResult {
    let name = bincode::deserialize::<String>(msg)
        .expect("a text function expects a deserializable utf-8 string");
    if name == "Panic" {
        panic!("The special panic name was invoked.");
    }
    let hello_text = format!("Hello, {}, this is a WASM app speaking!", name);
    let response =
        bincode::serialize(&hello_text).expect("should be able to bincode-serialize a string");
    Ok(response)
}
