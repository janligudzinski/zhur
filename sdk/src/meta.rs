use bincode::deserialize;
use wapc_guest::host_call;

/// Returns your username and your app's name as a tuple.
pub fn whoami() -> (String, String) {
    let whoami_bytes = host_call("", "internals", "whoami", &[]).unwrap();
    deserialize(&whoami_bytes).unwrap()
}
