/// This module contains crates the Zhur SDK uses, but you should not directly have to import.
pub mod __internals {
    pub use bincode;
    pub use wapc_guest;
}
/// This macro sets up all the Zhur SDK boilerplate for a text function that takes a UTF-8 `String` and returns another.
#[macro_export]
macro_rules! text_function {
    ($function_name:ident) => {
        fn func_wrapper(msg: &[u8]) -> zhur_sdk::__internals::wapc_guest::CallResult {
            let msg_string = zhur_sdk::__internals::bincode::deserialize::<String>(msg)
                .expect("a text function expects a deserializable utf-8 string");
            let output = $function_name(msg_string);
            let response = zhur_sdk::__internals::bincode::serialize(&output)
                .expect("should be able to bincode-serialize a string");
            Ok(response)
        }
        #[no_mangle]
        pub fn wapc_init() {
            std::panic::set_hook(Box::new(|p| {
                zhur_sdk::__internals::wapc_guest::host_call(
                    "zhur",
                    "internals",
                    "panic",
                    p.to_string().as_bytes(),
                )
                .unwrap();
            }));
            zhur_sdk::__internals::wapc_guest::register_function("text", func_wrapper)
        }
    };
}
