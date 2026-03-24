use crate::provider::{LsInstance, LsProvider, LsProviderConfig};
use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub struct MockLsInstance {
    grpc_addr: SocketAddr,
    csrf_token: Option<String>,
    identity: String,
    last_accessed: Mutex<std::time::Instant>,
    created_at: std::time::Instant,
}

impl Drop for MockLsInstance {
    fn drop(&mut self) {
        // Mock清理空操作
    }
}

impl LsInstance for MockLsInstance {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn grpc_addr(&self) -> SocketAddr {
        self.grpc_addr
    }

    fn csrf_token(&self) -> Option<String> {
        self.csrf_token.clone()
    }

    fn id(&self) -> String {
        format!("mock_{}", self.grpc_addr)
    }

    fn identity(&self) -> String {
        self.identity.clone()
    }

    fn creation_time(&self) -> std::time::Instant {
        self.created_at
    }

    fn last_accessed(&self) -> std::time::Instant {
        *self.last_accessed.lock().unwrap()
    }

    fn set_last_accessed(&self, time: std::time::Instant) {
        *self.last_accessed.lock().unwrap() = time;
    }
}

impl transcoder_core::common::ErrorFetcher for MockLsInstance {
    fn get_last_error(&self) -> Option<String> {
        None
    }
}

pub struct MockLsProvider {
    pub mocked_port: u16,
}

#[async_trait]
impl LsProvider for MockLsProvider {
    async fn acquire_instance(&self, identity: &str, _identity_token: &str, _slot_id: Option<&str>) -> Result<Arc<dyn LsInstance>> {
        Ok(Arc::new(MockLsInstance {
            grpc_addr: format!("127.0.0.1:{}", self.mocked_port).parse()?,
            // Mock 模式不需要 CSRF
            csrf_token: None,
            identity: identity.to_string(),
            last_accessed: Mutex::new(std::time::Instant::now()),
            created_at: std::time::Instant::now(),
        }))
    }

    async fn list_instances(&self) -> Result<Vec<crate::provider::InstanceInfo>> {
        let now = std::time::UNIX_EPOCH.elapsed()?.as_secs();
        Ok(vec![crate::provider::InstanceInfo {
            id: format!("mock_127.0.0.1:{}", self.mocked_port),
            identity: "mock@example.com".to_string(),
            grpc_addr: format!("127.0.0.1:{}", self.mocked_port),
            last_accessed_secs: now,
            created_at_secs: now,
            status: "active".to_string(),
        }])
    }

    async fn remove_instance(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }

    async fn get_config(&self) -> LsProviderConfig {
        LsProviderConfig::default()
    }

    async fn update_config(&self, _config: LsProviderConfig) -> Result<()> {
        Ok(())
    }
}
