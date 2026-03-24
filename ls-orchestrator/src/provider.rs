use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsProviderConfig {
    /// 允许的最大活跃实例数
    pub max_instances: usize,
    /// 实例空闲回收时长 (秒)，为 0 表示不回收
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub identity: String,
    pub grpc_addr: String,
    pub last_accessed_secs: u64,
    pub created_at_secs: u64,
    /// "active" = 内存中活跃实例, "orphan" = 磁盘残留沙盒（进程已退出）
    pub status: String,
}

impl Default for LsProviderConfig {
    fn default() -> Self {
        Self {
            max_instances: 5,
            idle_timeout_secs: 1800, // 默认 30 分钟
        }
    }
}

/// 表示一个激活状态的 Language Server 实例
pub trait LsInstance: transcoder_core::common::ErrorFetcher + Send + Sync {
    /// 支持向下转型以访问特定实现的字段 (如 NativeLsInstance)
    fn as_any(&self) -> &dyn std::any::Any;

    /// 获取当前实例监听的 gRPC 地址
    fn grpc_addr(&self) -> SocketAddr;
    
    /// 获取实例所需的鉴权参数 (如 token)
    fn csrf_token(&self) -> Option<String>;
    
    /// 获取实例在此提供者中的唯一 ID (例如 Hash)
    fn id(&self) -> String;

    /// 获取关联的身份标识 (例如 Email)
    fn identity(&self) -> String;

    /// 获取启动时间
    fn creation_time(&self) -> std::time::Instant;

    /// 获取最后一次被访问的时间（用于 LRU 淘汰）
    fn last_accessed(&self) -> std::time::Instant;

    /// 更新最后访问时间
    fn set_last_accessed(&self, time: std::time::Instant);
}

/// 提供创建/路由到指定实例的接口
#[async_trait]
pub trait LsProvider: Send + Sync {
    /// 尝试根据身份向提供者获取一个可用实例。提供可选的 slot_id 进行固定位置复用。
    /// 不同的环境（如 Docker, Local Native）可以用不同方式实现（如冷启动、池化复用）。
    async fn acquire_instance(&self, identity: &str, identity_token: &str, slot_id: Option<&str>) -> Result<Arc<dyn LsInstance>>;

    /// 列出当前挂载的所有活跃实例的详细信息
    async fn list_instances(&self) -> Result<Vec<InstanceInfo>>;

    /// 根据 ID 强制手动斩杀或移除特定的实例
    async fn remove_instance(&self, id: &str) -> Result<bool>;

    /// 获取当前的治理配置
    async fn get_config(&self) -> LsProviderConfig;

    /// 动态更新治理配置
    async fn update_config(&self, config: LsProviderConfig) -> Result<()>;
}
