pub use bincode;
pub use wapc_guest;
/// This macro sets up all the Zhur SDK boilerplate for a text function that takes a UTF-8 `String` and returns another.
#[macro_export]
macro_rules! text_function {
    ($function_name:ident) => {
        fn func_wrapper(msg: &[u8]) -> sdk::wapc_guest::CallResult {
            let msg_string = sdk::bincode::deserialize::<String>(msg)
                .expect("a text function expects a deserializable utf-8 string");
            let output = $function_name(msg_string);
            let response = sdk::bincode::serialize(&output)
                .expect("should be able to bincode-serialize a string");
            Ok(response)
        }
        #[no_mangle]
        pub fn wapc_init() {
            std::panic::set_hook(Box::new(|p| {
                sdk::wapc_guest::host_call("zhur", "internals", "panic", p.to_string().as_bytes())
                    .unwrap();
            }));
            sdk::wapc_guest::register_function("text", func_wrapper)
        }
    };
}
