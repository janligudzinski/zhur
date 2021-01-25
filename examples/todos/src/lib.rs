use zhur_sdk::*;

mod data;
mod todo;
mod routes;
mod router;
use router::route;

handle_http!(route);