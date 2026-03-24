use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use crate::constants::LS_METADATA_IDE_VERSION;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityVersionInfo {
    /// Transcoder 模拟的目标版本 (如 1.20.5)
    pub simulated_version: String,
    /// 本地机器安装的 App 版本
    pub local_app_version: Option<String>,
    /// 官方最新的最新版本
    pub remote_latest_version: Option<String>,
}

pub struct VersionManager;

impl VersionManager {
    /// 获取全量版本信息
    pub async fn get_all_version_info() -> AntigravityVersionInfo {
        let simulated_version = crate::common::get_runtime_version();
        let local_app_version = Self::detect_local_version();
        let remote_latest_version = Self::fetch_remote_version().await;

        AntigravityVersionInfo {
            simulated_version,
            local_app_version,
            remote_latest_version,
        }
    }

    /// 探测本地安装的 Antigravity 版本
    fn detect_local_version() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            let paths = [
                "/Applications/Antigravity.app/Contents/Info.plist",
                "/Applications/Cursor.app/Contents/Info.plist", // 兼容
            ];

            for path in &paths {
                if let Ok(content) = std::fs::read(path) {
                    if let Ok(plist) = plist::Value::from_reader(std::io::Cursor::new(content)) {
                        if let Some(dict) = plist.as_dictionary() {
                            if let Some(v) = dict.get("CFBundleShortVersionString").and_then(|v| v.as_string()) {
                                return Some(v.to_string());
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 逻辑通常在 AppData 下
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                let path = std::path::Path::new(&user_profile)
                    .join("AppData\\Local\\Programs\\Antigravity\\resources\\app\\package.json");
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                            return Some(v.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    async fn fetch_remote_version() -> Option<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
            .build()
            .ok()?;

        // 核心修正：使用官方自动更新 JSON 接口 (fetch_official_assets.sh 同款)
        // 网页版 releases 只是个 SPA 壳子，不包含版本号文本。
        let primary_url = "https://antigravity-auto-updater-974169037036.us-central1.run.app/releases";
        match client.get(primary_url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "identity")
            .send().await {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    if let Some(ver) = Self::extract_version(&text) {
                        info!("🌐 通过官方 Releases API 获取到远程版本: {}", ver);
                        return Some(ver);
                    }
                    warn!("⚠️ Releases API 已响应，但未能提取版本号 (内容前100字符: {:.100})", text);
                }
            }
            Err(e) => {
                warn!("⚠️ Releases API 访问失败: {:?}", e);
            }
        }

        None
    }

    /// 提取语义化版本号
    /// 策略: 宽泛匹配任意位置的版本号
    fn extract_version(text: &str) -> Option<String> {
        let re_loose = regex::Regex::new(r"(\d+\.\d+\.\d+)").ok()?;
        re_loose.captures(text).map(|caps| caps[1].to_string())
    }
}
