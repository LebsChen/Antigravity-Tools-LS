use cli_server::run_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 允许通过环境变量覆盖端口，默认使用 5173
    let port: Option<u16> = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok());
        
    run_server(port).await
}
