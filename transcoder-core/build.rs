use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut protos = Vec::new();
    if let Ok(entries) = fs::read_dir("proto") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "proto" {
                if let Some(path_str) = path.to_str() {
                    protos.push(path_str.to_string());
                }
            }
        }
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(out_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&protos, &["proto"])?;

    println!("cargo:rerun-if-changed=proto/");
    Ok(())
}
