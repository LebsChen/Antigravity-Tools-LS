use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub name: String,
    pub created_at: i64,
}

pub struct KeyManager {
    file_path: PathBuf,
    keys: RwLock<Vec<ApiKey>>,
}

impl KeyManager {
    pub async fn new(base_dir: PathBuf) -> Result<Self> {
        let file_path = base_dir.join("api_keys.json");
        let keys = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        } else {
            // 如果文件不存在，自动生成一个默认的 Admin Key 作为兜底
            let default_key = ApiKey {
                key: format!("sk-antigravity-{}", Uuid::new_v4().to_string().replace("-", "")),
                name: "Default Admin Key".to_string(),
                created_at: Utc::now().timestamp(),
            };
            let initial_keys = vec![default_key];
            let content = serde_json::to_string_pretty(&initial_keys)?;
            fs::write(&file_path, content)?;
            initial_keys
        };

        Ok(Self {
            file_path,
            keys: RwLock::new(keys),
        })
    }

    pub async fn is_valid(&self, key: &str) -> bool {
        let keys = self.keys.read().await;
        keys.iter().any(|k| k.key == key)
    }

    pub async fn list_keys(&self) -> Vec<ApiKey> {
        self.keys.read().await.clone()
    }

    pub async fn create_key(&self, name: String) -> Result<ApiKey> {
        let new_key = ApiKey {
            // 生成标准的 sk-xxx 格式
            key: format!("sk-antigravity-{}", Uuid::new_v4().to_string().replace("-", "")),
            name,
            created_at: Utc::now().timestamp(),
        };

        let mut keys = self.keys.write().await;
        keys.push(new_key.clone());
        self.save_to_disk(&keys)?;

        Ok(new_key)
    }

    pub async fn delete_key(&self, key: &str) -> Result<bool> {
        let mut keys = self.keys.write().await;
        let p_len = keys.len();
        keys.retain(|k| k.key != key);
        
        if keys.len() < p_len {
            self.save_to_disk(&keys)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn rename_key(&self, key: &str, new_name: String) -> Result<Option<ApiKey>> {
        let mut keys = self.keys.write().await;
        if let Some(k) = keys.iter_mut().find(|k| k.key == key) {
            k.name = new_name;
            let updated = k.clone();
            self.save_to_disk(&keys)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    /// 同时修改密钥的 key 值和/或名称
    pub async fn update_key(&self, old_key: &str, new_key: Option<String>, new_name: Option<String>) -> Result<Option<ApiKey>> {
        let mut keys = self.keys.write().await;
        
        // 如果要修改 key 值，先检查新 key 是否已存在
        if let Some(ref nk) = new_key {
            if keys.iter().any(|k| k.key == *nk && k.key != old_key) {
                return Err(anyhow::anyhow!("该 API Key 已存在，请使用其他值"));
            }
        }
        
        if let Some(k) = keys.iter_mut().find(|k| k.key == old_key) {
            if let Some(nk) = new_key {
                k.key = nk;
            }
            if let Some(nn) = new_name {
                k.name = nn;
            }
            let updated = k.clone();
            self.save_to_disk(&keys)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    fn save_to_disk(&self, keys: &[ApiKey]) -> Result<()> {
        let content = serde_json::to_string_pretty(keys)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }
}
