use std::path::Path;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Token 统计汇总条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatsEntry {
    pub period: String, 
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub request_count: u64,
}

/// 流量日志明细（用于监控与审计）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficLog {
    pub id: String,
    pub timestamp: i64,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration: u64, // ms
    pub model: Option<String>,
    pub mapped_model: Option<String>,
    pub account_email: Option<String>,
    pub client_ip: Option<String>,
    pub error: Option<String>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub protocol: String, // "openai", "anthropic", "gemini"
}

pub struct StatsManager {
    db_path: String,
}

impl StatsManager {
    pub fn new(data_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db_path = data_dir.as_ref().join("usage.db");
        let path_str = db_path.to_string_lossy().to_string();
        
        let conn = Connection::open(&db_path)?;
        
        // 初始化表结构
        conn.execute(
            "CREATE TABLE IF NOT EXISTS token_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                account TEXT NOT NULL,
                model TEXT NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS token_stats_hourly (
                hour_bucket TEXT NOT NULL,
                account TEXT NOT NULL,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                request_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (hour_bucket, account)
            )",
            [],
        )?;

        info!("Token stats database initialized successfully: {}", path_str);
        Ok(Self { db_path: path_str })
    }

    fn connect(&self) -> anyhow::Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        Ok(conn)
    }

    /// 记录一次使用数据
    pub fn record_usage(
        &self,
        account: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> anyhow::Result<()> {
        let conn = self.connect()?;
        let timestamp = chrono::Local::now().timestamp();
        let total = input_tokens + output_tokens;

        // 1. 插入明细
        conn.execute(
            "INSERT INTO token_usage (timestamp, account, model, input_tokens, output_tokens, total_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![timestamp, account, model, input_tokens, output_tokens, total],
        )?;

        // 2. 更新小时聚合
        let hour_bucket = chrono::Local::now().format("%Y-%m-%d %H:00").to_string();
        conn.execute(
            "INSERT INTO token_stats_hourly (hour_bucket, account, total_input_tokens, total_output_tokens, total_tokens, request_count)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)
             ON CONFLICT(hour_bucket, account) DO UPDATE SET
                total_input_tokens = total_input_tokens + ?3,
                total_output_tokens = total_output_tokens + ?4,
                total_tokens = total_tokens + ?5,
                request_count = request_count + 1",
            params![hour_bucket, account, input_tokens, output_tokens, total],
        )?;

        Ok(())
    }

    /// 获取最近 N 小时的趋势
    pub fn get_hourly_trends(&self, hours: i64) -> anyhow::Result<Vec<TokenStatsEntry>> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now() - chrono::Duration::hours(hours);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:00").to_string();

        let mut stmt = conn.prepare(
            "SELECT hour_bucket, 
                SUM(total_input_tokens), 
                SUM(total_output_tokens), 
                SUM(total_tokens), 
                SUM(request_count)
             FROM token_stats_hourly
             WHERE hour_bucket >= ?1
             GROUP BY hour_bucket
             ORDER BY hour_bucket ASC"
        )?;

        let rows = stmt.query_map([cutoff_str], |row| {
            Ok(TokenStatsEntry {
                period: row.get(0)?,
                total_input_tokens: row.get(1)?,
                total_output_tokens: row.get(2)?,
                total_tokens: row.get(3)?,
                request_count: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// 获取长周期 (天级) 的聚合趋势
    pub fn get_daily_trends(&self, days: i64) -> anyhow::Result<Vec<TokenStatsEntry>> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

        let mut stmt = conn.prepare(
            "SELECT substr(hour_bucket, 1, 10) as day_bucket, 
                SUM(total_input_tokens), 
                SUM(total_output_tokens), 
                SUM(total_tokens), 
                SUM(request_count)
             FROM token_stats_hourly
             WHERE substr(hour_bucket, 1, 10) >= ?1
             GROUP BY day_bucket
             ORDER BY day_bucket ASC"
        )?;

        let rows = stmt.query_map([cutoff_str], |row| {
            Ok(TokenStatsEntry {
                period: row.get(0)?,
                total_input_tokens: row.get(1)?,
                total_output_tokens: row.get(2)?,
                total_tokens: row.get(3)?,
                request_count: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// 获取摘要数据 (总计/活跃数)
    pub fn get_summary_stats(&self) -> anyhow::Result<serde_json::Value> {
        let conn = self.connect()?;
        let res: (u64, u64, u64, u64) = conn.query_row(
            "SELECT 
                COALESCE(SUM(total_input_tokens), 0), 
                COALESCE(SUM(total_output_tokens), 0), 
                COALESCE(SUM(total_tokens), 0), 
                COALESCE(SUM(request_count), 0) 
             FROM token_stats_hourly",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        )?;

        let unique_accounts: u64 = conn.query_row(
            "SELECT COUNT(DISTINCT account) FROM token_stats_hourly",
            [],
            |row| row.get(0)
        )?;

        Ok(serde_json::json!({
            "total_input_tokens": res.0,
            "total_output_tokens": res.1,
            "total_tokens": res.2,
            "total_requests": res.3,
            "unique_accounts": unique_accounts
        }))
    }

    /// 获取模型分布排行
    pub fn get_model_stats(&self, hours: i64) -> anyhow::Result<serde_json::Value> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now().timestamp() - (hours * 3600);
        
        let mut stmt = conn.prepare(
            "SELECT model, SUM(total_tokens) as total, COUNT(*) as count 
             FROM token_usage 
             WHERE timestamp >= ?1 
             GROUP BY model 
             ORDER BY total DESC"
        )?;

        let rows = stmt.query_map([cutoff], |row| {
            Ok(serde_json::json!({
                "model": row.get::<_, String>(0)?,
                "tokens": row.get::<_, u64>(1)?,
                "requests": row.get::<_, u64>(2)?
            }))
        })?;

        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(serde_json::json!(results))
    }

    /// 获取账号消费排行
    pub fn get_account_stats(&self, hours: i64) -> anyhow::Result<serde_json::Value> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now() - chrono::Duration::hours(hours);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:00").to_string();

        let mut stmt = conn.prepare(
            "SELECT account, SUM(total_tokens) as total, SUM(request_count) as count 
             FROM token_stats_hourly 
             WHERE hour_bucket >= ?1 
             GROUP BY account 
             ORDER BY total DESC LIMIT 10"
        )?;

        let rows = stmt.query_map([cutoff_str], |row| {
            Ok(serde_json::json!({
                "account": row.get::<_, String>(0)?,
                "tokens": row.get::<_, u64>(1)?,
                "requests": row.get::<_, u64>(2)?
            }))
        })?;

        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(serde_json::json!(results))
    }

    /// 获取模型在时间线上的分布 (用于堆叠面积图)
    pub fn get_model_trend_hourly(&self, hours: i64) -> anyhow::Result<serde_json::Value> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now().timestamp() - (hours * 3600);

        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m-%d %H:00', datetime(timestamp, 'unixepoch', 'localtime')) as hour_bucket,
                model, SUM(total_tokens)
             FROM token_usage
             WHERE timestamp >= ?1
             GROUP BY hour_bucket, model
             ORDER BY hour_bucket ASC"
        )?;

        let rows = stmt.query_map([cutoff], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, u64>(2)?))
        })?;

        let mut trend_map: std::collections::BTreeMap<String, std::collections::HashMap<String, u64>> = std::collections::BTreeMap::new();
        for row in rows {
            let (period, model, total) = row?;
            trend_map.entry(period).or_default().insert(model, total);
        }

        let mut results = Vec::new();
        for (period, data) in trend_map {
            let mut entry = serde_json::json!({ "period": period });
            for (m, v) in data {
                entry.as_object_mut().unwrap().insert(m, serde_json::json!(v));
            }
            results.push(entry);
        }

        Ok(serde_json::json!(results))
    }

    /// 获取模型在时间线上的分布 (长周期，按天)
    pub fn get_model_trend_daily(&self, days: i64) -> anyhow::Result<serde_json::Value> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now().timestamp() - (days * 24 * 3600);

        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m-%d', datetime(timestamp, 'unixepoch', 'localtime')) as day_bucket,
                model, SUM(total_tokens)
             FROM token_usage
             WHERE timestamp >= ?1
             GROUP BY day_bucket, model
             ORDER BY day_bucket ASC"
        )?;

        let rows = stmt.query_map([cutoff], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, u64>(2)?))
        })?;

        let mut trend_map: std::collections::BTreeMap<String, std::collections::HashMap<String, u64>> = std::collections::BTreeMap::new();
        for row in rows {
            let (period, model, total) = row?;
            trend_map.entry(period).or_default().insert(model, total);
        }

        let mut results = Vec::new();
        for (period, data) in trend_map {
            let mut entry = serde_json::json!({ "period": period });
            for (m, v) in data {
                entry.as_object_mut().unwrap().insert(m, serde_json::json!(v));
            }
            results.push(entry);
        }

        Ok(serde_json::json!(results))
    }

    /// 获取最近 1 小时的平均延迟 (ms)
    pub fn get_recent_latency(&self) -> anyhow::Result<u64> {
        let conn = self.connect()?;
        let cutoff = chrono::Local::now().timestamp() - 3600;
        
        let res: Option<f64> = conn.query_row(
            "SELECT AVG(duration) FROM (
                SELECT duration FROM token_usage 
                WHERE timestamp >= ?1 
                ORDER BY timestamp DESC LIMIT 100
             )",
            params![cutoff],
            |row| row.get(0)
        )?;

        Ok(res.unwrap_or(0.0) as u64)
    }
}

