use std::path::Path;
use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use serde::Serialize;
use tracing::{Event, Level, Subscriber};
use tracing::field::{Field, Visit};
use tracing_subscriber::{
    fmt, layer::{SubscriberExt, Context}, util::SubscriberInitExt, EnvFilter, Layer
};

/// 日志条目结构，兼容前端展示
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: i64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// 全局日志计数器
static LOG_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 内存日志环形缓冲区
#[derive(Clone)]
pub struct MemoryLogRing {
    logs: Arc<RwLock<VecDeque<LogEntry>>>,
    pub broadcast_tx: tokio::sync::broadcast::Sender<LogEntry>,
    max_lines: usize,
}

impl MemoryLogRing {
    pub fn new(max_lines: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(1024);
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(max_lines))),
            broadcast_tx: tx,
            max_lines,
        }
    }

    pub fn fetch_logs(&self) -> Vec<LogEntry> {
        let lock = self.logs.read().unwrap();
        lock.iter().cloned().collect()
    }

    pub fn clear(&self) {
        let mut lock = self.logs.write().unwrap();
        lock.clear();
    }

    pub fn push(&self, entry: LogEntry) {
        // 先发送广播，让活跃连接尽快收到
        let _ = self.broadcast_tx.send(entry.clone());

        let mut lock = self.logs.write().unwrap();
        if lock.len() >= self.max_lines {
            lock.pop_front();
        }
        lock.push_back(entry);
    }
}

/// 字段访问器，用于从 Tracing Event 中提取字段
struct FieldVisitor {
    message: Option<String>,
    fields: HashMap<String, String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: HashMap::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let value_str = format!("{:?}", value);
        if field.name() == "message" {
            self.message = Some(value_str.trim_matches('"').to_string());
        } else {
            self.fields.insert(field.name().to_string(), value_str);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(field.name().to_string(), value.to_string());
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }
}

/// 结构化日志桥接层
pub struct StructuredLogLayer {
    ring: MemoryLogRing,
}

impl StructuredLogLayer {
    pub fn new(ring: MemoryLogRing) -> Self {
        Self { ring }
    }
}

impl<S> Layer<S> for StructuredLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            Level::ERROR => "ERROR",
            Level::WARN => "WARN",
            Level::INFO => "INFO",
            Level::DEBUG => "DEBUG",
            Level::TRACE => "TRACE",
        };

        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_default();
        if message.is_empty() && visitor.fields.is_empty() {
            return;
        }

        let entry = LogEntry {
            id: LOG_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            timestamp: chrono::Local::now().timestamp_millis(),
            level: level.to_string(),
            target: metadata.target().to_string(),
            message,
            fields: visitor.fields,
        };

        self.ring.push(entry);
    }
}

/// 本地时区格式化器
struct LocalTimer;

impl fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

/// 自动清理与限制日志文件大小
pub fn maintain_log_file(log_file: &Path, max_size_bytes: u64) -> Result<(), String> {
    use std::fs;

    if !log_file.exists() {
        return Ok(());
    }

    if let Ok(metadata) = fs::metadata(log_file) {
        if metadata.len() > max_size_bytes {
            // 文件超过限制，将其截断并保留末尾 10% 的内容以维持上下文
            // 简单实现：为了不干扰当前句柄，直接清空最为稳妥。
            // 进阶实现可以尝试 rename 后重新生成。
            let _ = fs::OpenOptions::new().write(true).truncate(true).open(log_file);
        }
    }
    Ok(())
}

/// 初始化全局日志订阅器
pub fn init_logger(log_dir: impl AsRef<Path>, memory_max_lines: usize) -> MemoryLogRing {
    let log_path = log_dir.as_ref();
    if !log_path.exists() {
        let _ = std::fs::create_dir_all(log_path);
    }

    // 取消每日滚动，直接使用单一文件
    let file_appender = tracing_appender::rolling::never(log_path, "gateway.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(_guard));

    let mem_ring = MemoryLogRing::new(memory_max_lines);
    let structured_layer = StructuredLogLayer::new(mem_ring.clone());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    // 控制台图层：带颜色，本地时间
    let console_layer = fmt::layer()
        .with_ansi(true)
        .with_timer(LocalTimer)
        .with_target(false);

    // 文件图层：JSON 格式，每日滚动
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .json()
        .with_timer(LocalTimer)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .with(structured_layer)
        .try_init()
        .ok(); 

    // 启动后台任务监控 log 文件大小 (默认限制 50MB)
    let log_file_path = log_path.join("gateway.log");
    tokio::spawn(async move {
        loop {
            let _ = maintain_log_file(&log_file_path, 50 * 1024 * 1024);
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    mem_ring
}
