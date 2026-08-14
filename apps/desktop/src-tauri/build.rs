fn main() {
    // Unit tests exercise the Rust core without producing a desktop bundle. Keep
    // production resource parity strict, but let that explicit test-only switch
    // remove bundle resources from Tauri's in-memory merge-patched config.
    if std::env::var_os("RESEARCHLEDGER_SKIP_BUNDLE_RESOURCE_VALIDATION").is_some() {
        std::env::set_var("TAURI_CONFIG", r#"{"bundle":{"resources":null}}"#);
    }
    tauri_build::build()
}
