/// This is a very simple code-getter only intended for development use, which searches for compiled WASM files in the workspace directory. Panics if not found. Absolutely never use in production.
pub fn get_code_simple(_owner: &str, name: &str) -> Option<Vec<u8>> {
    let mut path = std::env::current_dir().expect("could not get current dir");
    path.push("wasm-examples");
    path.push("target");
    path.push("wasm32-unknown-unknown");
    let filename = format!("{}.{}", name, ".wasm");
    path.push(filename);
    let exists = std::path::Path::exists(&path);
    if !exists {
        return None;
    }
    let bytes = std::fs::read(path).expect("could not read the file");
    Some(bytes)
}
