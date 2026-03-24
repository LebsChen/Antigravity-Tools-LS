use axum::{
    body::Body,
    extract::{Request, State},
    response::Response,
    http::StatusCode,
    routing::any,
    Router,
};
use tokio::net::TcpListener;
use reqwest::Client;
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 代理服务的共享状态 — 持有账号管理器以支持动态解析 project_id
#[derive(Clone)]
pub struct ProxyState {
    pub account_manager: Arc<ls_accounts::AccountManager>,
    pub project_id: Arc<RwLock<Option<String>>>, // 保留作为回退 ID
}

// 专门处理流量劫持的 Handler，将 ls_core 请求原样转发并覆写 User-Agent
async fn proxy_handler(
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("");
    let url = if query.is_empty() {
        format!("{}{}", crate::constants::PROXY_UPSTREAM_HOST, path)
    } else {
        format!("{}{}?{}", crate::constants::PROXY_UPSTREAM_HOST, path, query)
    };

    let auth_token = req.headers().get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let method = req.method().clone();
    let mut headers = req.headers().clone();
    // 必须移除原始的 host，否则 reqwest 发送的目标 host 将错乱
    headers.remove(axum::http::header::HOST);
    // **重点** 移除压缩声明，强制让云端返回明文方便我们在本地捕获肉眼可见分析
    headers.remove(axum::http::header::ACCEPT_ENCODING);

    let client = Client::new();
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await.map_err(|_| StatusCode::BAD_REQUEST)?;

    let builder = client.request(method, url)
        .headers(headers)
        .body(body_bytes);

    match builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();

            // 🔑 拦截 loadCodeAssist 响应以注入正确的 project_id
            if path.contains("loadCodeAssist") {
                // 1. 尝试使用提前提取的 Token 定位项目 ID
                let mut project_id_to_inject = state.project_id.read().await.clone();
                
                if let Some(ref token) = auth_token {
                    if let Some(account_id) = state.account_manager.find_account_id_by_token(token).await {
                        if let Ok(Some(account)) = state.account_manager.get_account(&account_id).await {
                            if let Some(pid) = account.project_id {
                                info!("🔑 [Proxy] 为账号 {} 动态解析到 Project ID: {}", account.email, pid);
                                project_id_to_inject = Some(pid);
                            }
                        }
                    }
                }

                if let Some(ref project_id) = project_id_to_inject {
                    info!("🔑 [Proxy] 正在注入 Project ID: {} (路径: {})", project_id, path);
                    let resp_bytes = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
                    let mut json_val: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap_or_default();
                    
                    // 动态注入云端 AI 伴侣项目信息
                    json_val["cloudaicompanionProject"] = serde_json::json!(project_id);
                    json_val["project"] = serde_json::json!({"id": project_id, "projectId": project_id});
                    
                    let modified_body = serde_json::to_vec(&json_val).unwrap_or_default();
                    info!("🔑 [Proxy] 已将 Project ID 注入 loadCodeAssist 响应 (长度: {})", modified_body.len());
                    
                    let mut response = axum::response::Response::builder().status(status);
                    *response.headers_mut().unwrap() = resp_headers;
                    response.headers_mut().unwrap().remove("content-length");
                    return Ok(response.body(Body::from(modified_body)).unwrap());
                } else {
                    info!("⚠️ [Proxy] 无法获取可用 Project ID，按原样转发 loadCodeAssist 响应");
                }
            }

            // 🔑 Intercept fetchAdminControls: return empty success instead of forwarding
            if path.contains("fetchAdminControls") {
                info!("🔑 [Proxy] Intercepting fetchAdminControls - returning empty success");
                let empty_resp = serde_json::json!({});
                let body = serde_json::to_vec(&empty_resp).unwrap_or_default();
                let mut response = axum::response::Response::builder().status(StatusCode::OK);
                response.headers_mut().unwrap().insert("content-type", "application/json".parse().unwrap());
                return Ok(response.body(Body::from(body)).unwrap());
            }

            let mut response = axum::response::Response::builder().status(status);
            *response.headers_mut().unwrap() = resp_headers;
            
            let mut stream = resp.bytes_stream();
            let body = Body::from_stream(async_stream::stream! {
                use tokio_stream::StreamExt;
                while let Some(chunk) = stream.next().await {
                    yield chunk;
                }
            });
            Ok(response.body(body).unwrap())
        },
        Err(e) => {
            error!("内置代理网关路由失败: {}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

/// Resolve the project_id from the account manager.
/// Scans all active accounts and returns the first one with a project_id set.
pub async fn resolve_project_id(account_manager: &ls_accounts::AccountManager) -> Option<String> {
    let summaries = account_manager.list_accounts().await;
    for summary in summaries {
        if summary.status != ls_accounts::AccountStatus::Active {
            continue;
        }
        if let Ok(Some(account)) = account_manager.get_account(&summary.id).await {
            if let Some(pid) = account.project_id {
                return Some(pid);
            }
        }
    }
    None
}

/// 启动内联反代网关并返回其绑定端口
pub async fn start_inline_proxy(
    account_manager: Arc<ls_accounts::AccountManager>,
    project_id: Arc<RwLock<Option<String>>>,
    version: String,
) -> anyhow::Result<u16> {
    let state = ProxyState { account_manager, project_id };
    let app = Router::new()
        .route("/*path", any(proxy_handler))
        .with_state(state);
    
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    
    info!("🛡  内置反代网关已启动 (端口: {}, 拦截版本: {})", port, version);
    
    // 异步挂载到后台常驻
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("内置代理网关运行中崩溃: {}", e);
        }
    });
    
    Ok(port)
}
