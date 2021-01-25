/// This macro takes any function that takes an `&HttpReq` and a `&mut HttpRes`, then generates the WAPC boilerplate for running it.
/// This lets you just focus on writing your app.
/// HTTP request deserialization, response serialization and all such are all taken care of here.
#[macro_export]
macro_rules! handle_http {
    ($http_handler:ident) => {
        fn outer_handler(msg: &[u8]) -> zhur_sdk::reex::wapc_guest::CallResult {
            let output = inner_handler(msg);
            let bytes = zhur_sdk::reex::bincode::serialize(&output).unwrap(); // WE DO NOT EXPECT TO FAIL HERE
            Ok(bytes)
        }
        fn inner_handler(msg: &[u8]) -> zhur_sdk::http::HttpRes {
            let req: zhur_sdk::http::HttpReq = zhur_sdk::reex::bincode::deserialize(msg).unwrap(); // WE DO NOT EXPECT TO FAIL HERE
            let mut res = zhur_sdk::http::HttpRes::default();
            $http_handler(&req, &mut res);
            res
        }
        #[no_mangle]
        pub extern "C" fn wapc_init() {
            zhur_sdk::reex::wapc_guest::register_function("handle_http", outer_handler);
        }
    };
}