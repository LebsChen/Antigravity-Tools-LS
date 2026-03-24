use std::path::PathBuf;
use tokio::sync::RwLock;
use crate::model::{Account, AccountIndex, AccountSummary};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AccountManager {
    base_dir: PathBuf,
    index: RwLock<AccountIndex>,
    token_map: RwLock<HashMap<String, String>>,
    // 用于处理并发刷新 Token 的锁，Key 为 Account ID 或 Refresh Token 标识
    refresh_locks: RwLock<HashMap<String, Arc<Mutex<()>>>>,
}

impl AccountManager {
    pub async fn new(base_dir: PathBuf) -> Result<Self> {
        let accounts_dir = base_dir.join("accounts");
        if !accounts_dir.exists() {
            tokio::fs::create_dir_all(&accounts_dir).await?;
        }

        let index_path = base_dir.join("accounts.json");
        let index = if index_path.exists() {
            let content = tokio::fs::read_to_string(&index_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AccountIndex {
                version: "1.0".to_string(),
                accounts: Vec::new(),
                current_account_id: None,
            }
        };

        // 在服务启动时刻极速扫描，将 O(N) 全盘查询抽象为 O(1) Token-to-Id HashMap
        // 同时：自修复 AccountSummary 里的 Label 字段（如果索引里缺失）
        let mut t_map = HashMap::new();
        let mut index_changed = false;
        let mut accounts_index = index;
        
        for summary in &mut accounts_index.accounts {
            let account_path = base_dir.join("accounts").join(format!("{}.json", summary.id));
            if let Ok(content) = tokio::fs::read_to_string(&account_path).await {
                if let Ok(account) = serde_json::from_str::<Account>(&content) {
                    t_map.insert(account.token.access_token.clone(), account.id.clone());
                    t_map.insert(account.token.refresh_token.clone(), account.id.clone());
                    
                    // 同步标签与禁用状态
                    let mut summary_changed = false;
                    if summary.label != account.label {
                        summary.label = account.label.clone();
                        summary_changed = true;
                    }
                    if summary.is_proxy_disabled != account.is_proxy_disabled {
                        summary.is_proxy_disabled = account.is_proxy_disabled;
                        summary_changed = true;
                    }
                    
                    if summary_changed {
                        index_changed = true;
                    }
                }
            }
        }

        if index_changed {
            let index_path = base_dir.join("accounts.json");
            let index_content = serde_json::to_string_pretty(&accounts_index)?;
            tokio::fs::write(&index_path, index_content).await?;
        }

        Ok(Self {
            base_dir,
            index: RwLock::new(accounts_index),
            token_map: RwLock::new(t_map),
            refresh_locks: RwLock::new(HashMap::new()),
        })
    }

    pub async fn upsert_account(&self, account: Account) -> Result<()> {
        let account_id = account.id.clone();
        let email = account.email.clone();
        
        let at = account.token.access_token.clone();
        let rt = account.token.refresh_token.clone();

        // 1. 异步保存详细账号文件
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", account_id));
        let content = serde_json::to_string_pretty(&account)?;
        tokio::fs::write(&account_path, content).await?;

        // 2. 更新索引
        let mut index = self.index.write().await;
        let summary = AccountSummary {
            id: account_id.clone(),
            email: email.clone(),
            name: account.name.clone(),
            status: account.status.clone(),
            label: account.label.clone(),
            is_proxy_disabled: account.is_proxy_disabled,
            last_used: account.last_used,
        };

        if let Some(existing) = index.accounts.iter_mut().find(|a| a.id == account_id) {
            *existing = summary;
        } else {
            index.accounts.push(summary);
        }

        if index.current_account_id.is_none() {
            index.current_account_id = Some(account_id.clone());
        }

        // 3. 异步保存索引文件
        let index_path = self.base_dir.join("accounts.json");
        let index_content = serde_json::to_string_pretty(&*index)?;
        tokio::fs::write(&index_path, index_content).await?;
        
        // 4. 更新内存 Token Map (同时映射 RT 和 AT)
        let mut t_map = self.token_map.write().await;
        t_map.insert(at, account_id.clone());
        t_map.insert(rt, account_id.clone());

        Ok(())
    }

    pub async fn get_account(&self, id: &str) -> Result<Option<Account>> {
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        if !account_path.exists() {
            return Ok(None);
        }
        let content = tokio::fs::read_to_string(account_path).await?;
        let account = serde_json::from_str(&content)?;
        Ok(Some(account))
    }

    pub async fn list_accounts(&self) -> Vec<AccountSummary> {
        self.index.read().await.accounts.clone()
    }

    /// [优化] 原底层 O(N) 的磁盘查询已被优化为近 0 耗时的 HashMap .get
    pub async fn find_account_id_by_token(&self, access_token: &str) -> Option<String> {
        let t_map = self.token_map.read().await;
        t_map.get(access_token).cloned()
    }

    pub async fn remove_account(&self, id: &str) -> Result<bool> {
        // 先读取以便移除 Token
        if let Ok(Some(acc)) = self.get_account(id).await {
            let mut t_map = self.token_map.write().await;
            t_map.remove(&acc.token.access_token);
            t_map.remove(&acc.token.refresh_token);
        }

        // 1. 删除账号详细文件 (异步)
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        if account_path.exists() {
            tokio::fs::remove_file(account_path).await?;
        }

        // 2. 更新内存索引
        let mut index = self.index.write().await;
        let mut found = false;
        
        if let Some(pos) = index.accounts.iter().position(|a| a.id == id) {
            index.accounts.remove(pos);
            found = true;
        }

        if index.current_account_id.as_deref() == Some(id) {
            index.current_account_id = index.accounts.first().map(|a| a.id.clone());
        }

        if found {
            // 3. 同步保存索引文件 (异步)
            let index_path = self.base_dir.join("accounts.json");
            let index_content = serde_json::to_string_pretty(&*index)?;
            tokio::fs::write(&index_path, index_content).await?;
        }

        Ok(found)
    }

    pub async fn update_quota(&self, id: &str, quota: crate::model::QuotaData) -> Result<()> {
        let mut account = match self.get_account(id).await? {
            Some(a) => a,
            None => anyhow::bail!("Account not found"),
        };

        account.quota = Some(quota);
        account.last_used = chrono::Utc::now().timestamp();

        // 异步保存详细账号文件
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&account)?;
        tokio::fs::write(account_path, content).await?;

        Ok(())
    }

    pub async fn update_project_id(&self, id: &str, project_id: String) -> Result<()> {
        let mut account = match self.get_account(id).await? {
            Some(a) => a,
            None => anyhow::bail!("Account not found"),
        };

        account.project_id = Some(project_id);

        // 异步保存详细账号文件
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&account)?;
        tokio::fs::write(account_path, content).await?;

        Ok(())
    }

    /// 根据 refresh_token 字符串查找对应的 access_token (如果存在)
    /// [优化] O(N) 性能终结
    pub async fn find_account_id_by_token_str(&self, raw_token: &str) -> Option<String> {
        let map = self.token_map.read().await;
        if let Some(id) = map.get(raw_token) {
            if let Ok(Some(acc)) = self.get_account(id).await {
                return Some(acc.token.access_token);
            }
        }
        None
    }

    /// 智能路由：获取当前最适合使用的账号 (简单轮询/最少使用)
    pub async fn get_best_account(&self) -> Result<Option<Account>> {
        let mut index = self.index.write().await;
        if index.accounts.is_empty() {
            return Ok(None);
        }

        // 过滤掉 Forbidden 或手动禁用的账号
        let candidates: Vec<String> = index.accounts.iter()
            .filter(|a| a.status == crate::model::AccountStatus::Active && !a.is_proxy_disabled)
            .map(|a| a.id.clone())
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        // 策略：简单循环，记录上一次使用的位置
        let current_id = index.current_account_id.clone().unwrap_or_else(|| candidates[0].clone());
        let pos = candidates.iter().position(|id| id == &current_id).unwrap_or(0);
        let next_pos = (pos + 1) % candidates.len();
        let next_id = candidates[next_pos].clone();
        
        index.current_account_id = Some(next_id.clone());
        
        // 更新汇总信息里的最后使用时间 (仅内存)
        if let Some(summary) = index.accounts.iter_mut().find(|a| a.id == next_id) {
            summary.last_used = chrono::Utc::now().timestamp();
        }

        // [优化] 移除了原有的不必要的同步执行 fs::write(index_path) 逻辑，避免了轮询高并发时的 IO 瓶颈。
        
        self.get_account(&next_id).await
    }

    /// 获取针对特定账号的刷新互斥锁
    pub async fn get_refresh_lock(&self, account_identifier: &str) -> Arc<Mutex<()>> {
        let mut locks = self.refresh_locks.write().await;
        if let Some(lock) = locks.get(account_identifier) {
            return lock.clone();
        }
        let lock = Arc::new(Mutex::new(()));
        locks.insert(account_identifier.to_string(), lock.clone());
        lock
    }

    pub async fn update_label(&self, id: &str, label: Option<String>) -> Result<()> {
        let mut account = match self.get_account(id).await? {
            Some(a) => a,
            None => anyhow::bail!("Account not found"),
        };

        account.label = label.clone();

        // 1. 异步保存详细账号文件
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&account)?;
        tokio::fs::write(&account_path, content).await?;

        // 2. 同步更新索引中的标签
        let mut index = self.index.write().await;
        if let Some(existing) = index.accounts.iter_mut().find(|a| a.id == id) {
            existing.label = label;
            
            // 3. 同时保存索引文件
            let index_path = self.base_dir.join("accounts.json");
            let index_content = serde_json::to_string_pretty(&*index)?;
            tokio::fs::write(&index_path, index_content).await?;
        }

        Ok(())
    }

    pub async fn update_proxy_disabled(&self, id: &str, disabled: bool) -> Result<()> {
        let mut account = match self.get_account(id).await? {
            Some(a) => a,
            None => anyhow::bail!("Account not found"),
        };

        account.is_proxy_disabled = disabled;
        
        // 🚀 增强：支持手动恢复活跃状态
        if !disabled {
            // 如果用户手动启用，且当前处于 Forbidden 状态，则恢复为 Active
            if account.status == crate::model::AccountStatus::Forbidden {
                account.status = crate::model::AccountStatus::Active;
            }
            
            // 同时恢复 Quota 中的封禁标记
            if let Some(ref mut q) = account.quota {
                q.is_forbidden = false;
            }
        }

        // 1. 异步保存详细账号文件
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&account)?;
        tokio::fs::write(&account_path, content).await?;

        // 2. 同步更新索引中的禁用状态与状态
        let mut index = self.index.write().await;
        if let Some(existing) = index.accounts.iter_mut().find(|a| a.id == id) {
            existing.is_proxy_disabled = disabled;
            existing.status = account.status.clone();
            existing.label = account.label.clone();
            
            // 3. 同时保存索引文件
            let index_path = self.base_dir.join("accounts.json");
            let index_content = serde_json::to_string_pretty(&*index)?;
            tokio::fs::write(&index_path, index_content).await?;
        }

        Ok(())
    }

    /// 🚀 增强：支持账号列表重新排序
    pub async fn reorder_accounts(&self, account_ids: Vec<String>) -> Result<()> {
        let mut index = self.index.write().await;
        
        // 1. 创建 ID 到 Summary 的映射并清空原列表
        let mut account_map: HashMap<String, AccountSummary> = index.accounts
            .drain(..)
            .map(|a| (a.id.clone(), a))
            .collect();
        
        // 2. 按新顺序重新填充 accounts
        for id in account_ids {
            if let Some(summary) = account_map.remove(&id) {
                index.accounts.push(summary);
            }
        }
        
        // 3. 将剩余的账号（如果有的话）追加到末尾，确保数据不丢失
        for (_, summary) in account_map {
            index.accounts.push(summary);
        }
        
        // 4. 保存索引文件
        let index_path = self.base_dir.join("accounts.json");
        let index_content = serde_json::to_string_pretty(&*index)?;
        tokio::fs::write(&index_path, index_content).await?;
        
        Ok(())
    }

    /// 🚀 增强：将账号标记为封禁状态
    /// 包括：状态设为 Forbidden, 禁用代理, 以及在标签中添加 [403 Forbidden]
    pub async fn mark_account_as_forbidden(&self, id: &str, reason: &str, appeal_url: Option<String>) -> Result<()> {
        let mut account = match self.get_account(id).await? {
            Some(a) => a,
            None => anyhow::bail!("Account not found"),
        };

        account.status = crate::model::AccountStatus::Forbidden;
        account.is_proxy_disabled = true;
        
        // 🚀 核心修复：即使 Quota 是 None，也要初始化对象以存储封禁原因和申诉链接
        let quota = account.quota.get_or_insert_with(|| crate::model::QuotaData {
            last_updated: chrono::Utc::now().timestamp(),
            ..Default::default()
        });
        
        quota.is_forbidden = true;
        quota.forbidden_reason = Some(reason.to_string());
        
        if let Some(url) = appeal_url {
            quota.extra.insert("appeal_url".to_string(), serde_json::Value::String(url));
        }
        
        account.disabled_reason = Some(reason.to_string());

        // 1. 保存详细文件
        let account_path = self.base_dir.join("accounts").join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&account)?;
        tokio::fs::write(&account_path, content).await?;

        // 2. 更新内存索引并同步
        let mut index = self.index.write().await;
        if let Some(existing) = index.accounts.iter_mut().find(|a| a.id == id) {
            existing.status = account.status.clone();
            existing.is_proxy_disabled = true;
            existing.label = account.label.clone();
            
            // 3. 保存索引
            let index_path = self.base_dir.join("accounts.json");
            let index_content = serde_json::to_string_pretty(&*index)?;
            tokio::fs::write(&index_path, index_content).await?;
        }

        Ok(())
    }
}
