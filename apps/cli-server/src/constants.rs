/// 控制常规外联的默认面孔 (User-Agent)
pub const DEFAULT_USER_AGENT: &str = "vscode/1.X.X (Antigravity/4.1.28)";

/// 上游的 Google API 域名
pub const PROXY_UPSTREAM_HOST: &str = "https://daily-cloudcode-pa.googleapis.com";

/// Google OAuth 授权界面的基础 URL
pub const OAUTH_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

/// Google OAuth Token 换取/刷新 URL
pub const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

// --- LS Stdin 注入元数据配置 ---
pub use transcoder_core::constants::*;
