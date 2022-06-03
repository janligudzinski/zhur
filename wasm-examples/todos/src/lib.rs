use zhur_sdk::*;

mod data;
mod router;
mod routes;
mod todo;
use router::route;

http_function!(route);
