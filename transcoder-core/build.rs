use std::env;
use std::fs;
use std::path::PathBuf;
use protox::prost::Message;

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

    // 使用纯 Rust 实现的 protox 编译 proto 文件
    // 无需系统安装 protoc，内置 Well-Known Types 和 proto3 optional 支持
    let fds = protox::compile(&protos, &["proto"])?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(out_dir)
        .compile_fds(fds)?;

    println!("cargo:rerun-if-changed=proto/");
    Ok(())
}
