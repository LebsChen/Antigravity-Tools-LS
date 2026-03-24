use axum::{
    extract::{State, Path},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;

use crate::state::AppState;
use crate::handlers::{ErrorResponse, ErrorDetail};

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RenameKeyRequest {
    pub name: Option<String>,
    pub key: Option<String>,
}

pub async fn list_keys_api(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let keys = state.key_manager.list_keys().await;
    (StatusCode::OK, Json(keys))
}

pub async fn create_key_api(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateKeyRequest>,
) -> impl IntoResponse {
    match state.key_manager.create_key(payload.name).await {
        Ok(key) => (StatusCode::CREATED, Json(key)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail { message: format!("创建 Key 失败: {}", e) }
            })
        ).into_response()
    }
}

pub async fn delete_key_api(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match state.key_manager.delete_key(&key).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail { message: "未找到指定的 Key".into() }
            })
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail { message: format!("删除 Key 失败: {}", e) }
            })
        ).into_response()
    }
}

pub async fn rename_key_api(
    State(state): State<Arc<AppState>>,
    Path(old_key): Path<String>,
    Json(payload): Json<RenameKeyRequest>,
) -> impl IntoResponse {
    match state.key_manager.update_key(&old_key, payload.key, payload.name).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail { message: "未找到指定的 Key".into() }
            })
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail { message: format!("更新 Key 失败: {}", e) }
            })
        ).into_response()
    }
}
