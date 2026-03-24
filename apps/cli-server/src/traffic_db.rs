use transcoder_core::transcoder::TrafficLog;
use std::path::Path;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct TrafficManager {
    db_conn: Arc<Mutex<Connection>>,
    pub broadcast_tx: broadcast::Sender<TrafficLog>,
}

impl TrafficManager {
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let db_path = data_dir.as_ref().join("traffic.db");
        let conn = Connection::open(db_path)?;

        // 初始化表结构
        conn.execute(
            "CREATE TABLE IF NOT EXISTS traffic_logs (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER NOT NULL,
                duration INTEGER NOT NULL,
                model TEXT,
                mapped_model TEXT,
                account_email TEXT,
                client_ip TEXT,
                error TEXT,
                input_tokens INTEGER,
                output_tokens INTEGER,
                protocol TEXT NOT NULL
            )",
            [],
        )?;

        // 为常用查询字段建立索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_traffic_timestamp ON traffic_logs (timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_traffic_email ON traffic_logs (account_email)", [])?;

        let (broadcast_tx, _) = broadcast::channel(100);

        Ok(Self {
            db_conn: Arc::new(Mutex::new(conn)),
            broadcast_tx,
        })
    }

    pub fn record_log(&self, log: TrafficLog) -> Result<()> {
        // 先发送广播，保证实时监控的极速响应
        let _ = self.broadcast_tx.send(log.clone());

        // 异步写入数据库 (这里我们可以选择同步写，因为 rusqlite 默认是同步的，
        // 建议在外部使用 tokio::task::spawn_blocking 包装此调用)
        let conn = self.db_conn.lock().unwrap();
        conn.execute(
            "INSERT INTO traffic_logs (
                id, timestamp, method, url, status, duration, 
                model, mapped_model, account_email, client_ip, 
                error, input_tokens, output_tokens, protocol
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                log.id,
                log.timestamp,
                log.method,
                log.url,
                log.status,
                log.duration,
                log.model,
                log.mapped_model,
                log.account_email,
                log.client_ip,
                log.error,
                log.input_tokens,
                log.output_tokens,
                log.protocol,
            ],
        )?;

        Ok(())
    }

    pub fn get_recent_logs(&self, limit: usize, offset: usize) -> Result<Vec<TrafficLog>> {
        let conn = self.db_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, method, url, status, duration, 
                    model, mapped_model, account_email, client_ip, 
                    error, input_tokens, output_tokens, protocol 
             FROM traffic_logs 
             ORDER BY timestamp DESC 
             LIMIT ?1 OFFSET ?2",
        )?;

        let logs = stmt.query_map(params![limit, offset], |row| {
            Ok(TrafficLog {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                method: row.get(2)?,
                url: row.get(3)?,
                status: row.get(4)?,
                duration: row.get(5)?,
                model: row.get(6)?,
                mapped_model: row.get(7)?,
                account_email: row.get(8)?,
                client_ip: row.get(9)?,
                error: row.get(10)?,
                input_tokens: row.get(11)?,
                output_tokens: row.get(12)?,
                protocol: row.get(13)?,
            })
        })?;

        let mut result = Vec::new();
        for log in logs {
            result.push(log?);
        }
        Ok(result)
    }

    pub fn cleanup_old_logs(&self, days: u32) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp_millis() - (days as i64 * 24 * 60 * 60 * 1000);
        let conn = self.db_conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM traffic_logs WHERE timestamp < ?1", params![cutoff])?;
        Ok(deleted)
    }

    pub fn clear_all_logs(&self) -> Result<()> {
        let conn = self.db_conn.lock().unwrap();
        conn.execute("DELETE FROM traffic_logs", [])?;
        conn.execute("VACUUM", [])?;
        Ok(())
    }
}
