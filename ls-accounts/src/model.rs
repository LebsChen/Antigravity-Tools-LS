use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 设备指纹（与 IDE 中的 storage.json 字段对应）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceProfile {
    pub machine_id: String,
    pub mac_machine_id: String,
    pub dev_device_id: String,
    pub sqm_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountStatus {
    Active,
    Expired,
    Forbidden,
    RateLimited,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQuota {
    pub name: String,
    pub percentage: i32,  // 剩余百分比 0-100
    pub reset_time: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_thinking_budget: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_video: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_mime_types: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuotaData {
    pub models: Vec<ModelQuota>,
    pub last_updated: i64,
    #[serde(default)]
    pub is_forbidden: bool,
    #[serde(default)]
    pub forbidden_reason: Option<String>,
    #[serde(default)]
    pub subscription_tier: Option<String>,
    #[serde(default)]
    pub model_forwarding_rules: HashMap<String, String>,
    
    /// 捕获所有其它未定义的字段（如 agentModelSorts, tabModelIds 等）
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String, // 通常是 refresh_token 的 MD5 或唯一哈希
    pub email: String,
    pub name: Option<String>,
    pub token: OAuthToken,
    pub status: AccountStatus,
    pub disabled_reason: Option<String>,
    pub project_id: Option<String>,
    pub label: Option<String>,
    #[serde(default)]
    pub is_proxy_disabled: bool,
    pub created_at: i64,
    pub last_used: i64,
    #[serde(default)]
    pub quota: Option<QuotaData>,
    /// 绑定的设备指纹，确保切换账号时 IDE 识别为同一设备
    #[serde(default)]
    pub device_profile: Option<DeviceProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub status: AccountStatus,
    pub label: Option<String>,
    #[serde(default)]
    pub is_proxy_disabled: bool,
    pub last_used: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountIndex {
    pub version: String,
    pub accounts: Vec<AccountSummary>,
    pub current_account_id: Option<String>,
}
