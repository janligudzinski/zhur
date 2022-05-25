use bincode::deserialize;
pub use chrono;
use wapc_guest::host_call;

/// Gets a UTC `chrono` timestamp of the current moment.
pub fn now() -> chrono::NaiveDateTime {
    let ndt_bytes = host_call("", "", "datetime", &[]).unwrap();
    deserialize(&ndt_bytes).unwrap()
}
