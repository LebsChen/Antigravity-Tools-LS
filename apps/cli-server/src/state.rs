use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashSet;
use ls_orchestrator::provider::LsProvider;
use crate::key_manager::KeyManager;
use transcoder_core::transcoder::StatsManager;
use crate::handlers::settings::AppSettings;

// 应用全局状态
pub struct AppState {
    pub provider: RwLock<Option<Arc<dyn LsProvider>>>,
    pub account_manager: Arc<ls_accounts::AccountManager>,
    pub tls_cert: RwLock<Option<Vec<u8>>>,
    pub http_client: reqwest::Client,
    pub port: u16,

    // 活跃的授权 State 缓存 (一次性校验)
    pub auth_states: RwLock<HashSet<String>>,

    // 内存中的近实时日志缓冲池，用于在 Web 页面显示
    pub mem_logger: crate::logger::MemoryLogRing,

    // 全局 API Key 管理器
    pub key_manager: Arc<KeyManager>,

    // Token 统计管理器
    pub stats_mgr: Arc<StatsManager>,

    // 流量日志管理器
    pub traffic_mgr: Arc<crate::traffic_db::TrafficManager>,

    // 全局应用配置（内存中热数据，可通过 /v1/settings 读写）
    pub app_settings: RwLock<AppSettings>,

    // 底层内核同步进度广播 (用于资源下载进度)
    pub sync_tx: tokio::sync::broadcast::Sender<crate::handlers::provision::SyncProgressEvent>,

    // 账号变更事件广播 (用于实时通知 UI 刷新)
    pub account_tx: tokio::sync::broadcast::Sender<String>,
}
