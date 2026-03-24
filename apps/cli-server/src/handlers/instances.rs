use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::state::AppState;
use ls_orchestrator::provider::LsProvider;

pub async fn get_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let provider_guard = state.provider.read().await;
    let provider = match provider_guard.as_ref() {
        Some(p) => p.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "核心内核初始化中").into_response(),
    };
    drop(provider_guard);

    match provider.list_instances().await {
        Ok(infos) => {
            if let Some(info) = infos.into_iter().find(|i| i.id == id) {
                (StatusCode::OK, Json(info)).into_response()
            } else {
                (StatusCode::NOT_FOUND, "实例不存在").into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn list_instances(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let provider_guard = state.provider.read().await;
    let provider = match provider_guard.as_ref() {
        Some(p) => p.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "核心内核初始化中").into_response(),
    };
    drop(provider_guard);

    match provider.list_instances().await {
        Ok(infos) => (StatusCode::OK, Json(infos)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn remove_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let provider_guard = state.provider.read().await;
    let provider = match provider_guard.as_ref() {
        Some(p) => p.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "核心内核初始化中").into_response(),
    };
    drop(provider_guard);

    match provider.remove_instance(&id).await {
        Ok(true) => (StatusCode::OK, "实例已下线").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "实例不存在").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("移除失败: {}", e)).into_response(),
    }
}

pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let provider_guard = state.provider.read().await;
    let provider = match provider_guard.as_ref() {
        Some(p) => p.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "核心内核初始化中").into_response(),
    };
    drop(provider_guard);

    let cfg = provider.get_config().await;
    Json(cfg).into_response()
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ls_orchestrator::provider::LsProviderConfig>,
) -> impl IntoResponse {
    let provider_guard = state.provider.read().await;
    let provider = match provider_guard.as_ref() {
        Some(p) => p.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "核心内核初始化中").into_response(),
    };
    drop(provider_guard);

    match provider.update_config(config).await {
        Ok(_) => (StatusCode::OK, "配置更新成功").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("更新失败: {}", e)).into_response(),
    }
}
