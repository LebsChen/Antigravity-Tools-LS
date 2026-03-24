use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde_json::json;

use crate::state::AppState;

/// GET /v1/logs
/// 返回当前驻留在内存中的结构化日志。
/// 支持前端进行级别过滤、字段显示等高级操作。
pub async fn fetch_memory_logs_api(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let logs = state.mem_logger.fetch_logs();
    
    Json(json!({
        "total": logs.len(),
        "lines": logs
    }))
}

/// DELETE /v1/logs
/// 清空当前驻留在内存中的结构化日志。
pub async fn clear_memory_logs_api(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.mem_logger.clear();
    axum::http::StatusCode::NO_CONTENT
}
