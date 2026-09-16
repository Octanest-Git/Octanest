fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/runner.proto");
    // Prefer PATH / PROTOC; fall back to user-local protoc cache (no sudo required).
    if std::env::var_os("PROTOC").is_none() {
        let home = std::env::var("HOME").unwrap_or_default();
        for ver in ["29.3", "28.3", "27.1"] {
            let candidate = format!("{home}/.cache/protoc-{ver}/bin/protoc");
            if std::path::Path::new(&candidate).is_file() {
                std::env::set_var("PROTOC", candidate);
                break;
            }
        }
    }
    prost_build::compile_protos(&["proto/runner.proto"], &["proto/"])?;
    Ok(())
}
