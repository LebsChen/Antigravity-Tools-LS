use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde_json::json;
use sysinfo::{System, RefreshKind, ProcessRefreshKind, MemoryRefreshKind};

use crate::state::AppState;

/// GET /v1/stats/hourly
/// GET /v1/stats/hourly?hours=24
pub async fn get_hourly_stats_api(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, i64>>,
) -> impl IntoResponse {
    let hours = params.get("hours").cloned().unwrap_or(24);
    match state.stats_mgr.get_hourly_trends(hours) {
        Ok(trends) => Json(json!({ "success": true, "data": trends })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/daily?days=7
pub async fn get_daily_stats_api(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, i64>>,
) -> impl IntoResponse {
    let days = params.get("days").cloned().unwrap_or(7);
    match state.stats_mgr.get_daily_trends(days) {
        Ok(trends) => Json(json!({ "success": true, "data": trends })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/summary
pub async fn get_summary_stats_api(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.stats_mgr.get_summary_stats() {
        Ok(summary) => Json(json!({ "success": true, "data": summary })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/models?hours=24
pub async fn get_model_stats_api(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, i64>>,
) -> impl IntoResponse {
    let hours = params.get("hours").cloned().unwrap_or(24);
    match state.stats_mgr.get_model_stats(hours) {
        Ok(stats) => Json(json!({ "success": true, "data": stats })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/accounts?hours=24
pub async fn get_account_stats_api(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, i64>>,
) -> impl IntoResponse {
    let hours = params.get("hours").cloned().unwrap_or(24);
    match state.stats_mgr.get_account_stats(hours) {
        Ok(stats) => Json(json!({ "success": true, "data": stats })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/model-trends?hours=24
pub async fn get_model_trends_api(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, i64>>,
) -> impl IntoResponse {
    let hours = params.get("hours").cloned().unwrap_or(24);
    match state.stats_mgr.get_model_trend_hourly(hours) {
        Ok(trends) => Json(json!({ "success": true, "data": trends })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/model-trends-daily?days=7
pub async fn get_model_trends_daily_api(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, i64>>,
) -> impl IntoResponse {
    let days = params.get("days").cloned().unwrap_or(7);
    match state.stats_mgr.get_model_trend_daily(days) {
        Ok(trends) => Json(json!({ "success": true, "data": trends })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /v1/stats/metrics — 获取系统与 LS 核心指标 (内存/压力)
pub async fn get_system_metrics_api(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::nothing().with_memory())
    );
    sys.refresh_all();
    
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    
    // 聚合所有名为 ls_core 的进程内存 (RSS)
    let mut ls_total_rss = 0u64;
    let mut ls_process_count = 0u32;
    
    for process in sys.processes().values() {
        let name = process.name().to_string_lossy();
        if name.contains("ls_core") || name.contains("vscodium") || name.contains("helper") {
            ls_total_rss += process.memory();
            ls_process_count += 1;
        }
    }

    let latency = state.stats_mgr.get_recent_latency().unwrap_or(0);
    let memory_pressure = if total_mem > 0 { (used_mem as f64 / total_mem as f64) * 100.0 } else { 0.0 };
    
    // 判定状态：检查 Provider 是否已热激活
    let provider_ready = state.provider.read().await.is_some();

    let status = if !provider_ready {
        "initializing"
    } else if ls_process_count > 0 && memory_pressure < 95.0 {
        "optimal"
    } else {
        "degraded"
    };

    Json(json!({
        "success": true,
        "data": {
            "status": status,
            "system_total_memory": total_mem,
            "system_used_memory": used_mem,
            "ls_aggregate_rss": ls_total_rss,
            "ls_process_count": ls_process_count,
            "latency_ms": latency,
            "memory_pressure_index": memory_pressure,
            "timestamp": chrono::Local::now().timestamp()
        }
    }))
}

