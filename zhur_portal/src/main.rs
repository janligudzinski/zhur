use std::net::{Ipv4Addr, SocketAddrV4};

use warp::filters;
use warp::Filter;
#[tokio::main]
async fn main() {
    let hw_filter = filters::method::get()
    .and(filters::path::end())
    .map(|| "Hello, world!".to_string());
    let hw_name_filter = filters::method::get()
    .and(filters::path::path("hello"))
    .and(filters::path::param())
    .map(|name: String| format!("Hello, {}!", name));
    let filter = hw_filter.or(hw_name_filter);
    warp::serve(filter).run(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8004)).await;
}