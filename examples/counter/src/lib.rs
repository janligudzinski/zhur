use zhur_sdk::{
    *,
    web::*,
    http::*,
};

fn counter(_req: &HttpReq, res: &mut HttpRes) {
    let counter = svc::kv::kv_get("counter_app", "counter").unwrap_or(0);
    let text = match counter {
        0 => "This app has never been run before!".into(),
        1 => "This app has been run once.".into(),
        _ => format!("This app has been run {} times!", counter)
    };
    svc::kv::kv_set("counter_app", "counter", &(counter + 1));
    Text(text).modify_response(res);
}
handle_http!(counter);