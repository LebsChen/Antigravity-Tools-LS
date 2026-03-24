
use transcoder_core::proto::google::internal::cloud::code::v1internal::{LoadCodeAssistRequest, ClientMetadata};
use transcoder_core::proto::exa::language_server_pb::language_server_service_client::LanguageServerServiceClient;
use tonic::{transport::Channel, Request};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用一个有效的 Access Token
    let token = "ya29.a0ATkoCc6rRXYYD3fE7Dj_nKmUxYySv5vAb9flakrvIYoNJfRB-jIxjuxWfZvl4txPVoE2VHpmHHMuXEFRfP354AgdQJEHwb4nobYp43VmiqmVNgFRgcuio9jkZW1oCTXGzfFa1o5FD-8JGXoFd_gqiHT199TMeLdoXgQSwTcCDTwu4ksry4J8R7Weak8r8UQPrvtRfd5rY-TaVaoaCgYKAYMSARYSFQHGX2Mi5U6e7GUWouTWg7wJm1Bx7g0214";
    
    let channel = Channel::from_static("https://daily-cloudcode-pa.googleapis.com").connect().await?;
    let _client = LanguageServerServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", format!("Bearer {}", token).parse().unwrap());
        // 关键：必须添加 IDE Info 让后端认可
        req.metadata_mut().insert("x-goog-api-client", "gl-go/1.27.0-20260209-RC00 gdcl/0.0.0".parse().unwrap());
        Ok(req)
    });

    let _request = Request::new(LoadCodeAssistRequest {
        cloudaicompanion_project: None,
        metadata: Some(ClientMetadata {
            ide_name: "antigravity".to_string(),
            ide_version: "1.19.6".to_string(),
            plugin_version: "1.19.6".to_string(),
            ide_type: 9, // ANTIGRAVITY
            ..Default::default()
        }),
        mode: None,
    });

    // 检查 LanguageServerServiceClient 是否真的没有这个方法
    // 根据之前的错误，它确实没有。我将尝试直接使用 Channel 来探测。
    
    Ok(())
}
