use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UpdateLabelReq {
    pub label: Option<String>,
}

pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let summaries = state.account_manager.list_accounts().await;
    (StatusCode::OK, Json(summaries)).into_response()
}

pub async fn get_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.account_manager.get_account(&id).await {
        Ok(Some(acc)) => (StatusCode::OK, Json(acc)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Account not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.account_manager.remove_account(&id).await {
        Ok(_) => {
            let _ = state.account_tx.send("deleted".to_string());
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_account_label(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateLabelReq>,
) -> impl IntoResponse {
    match state.account_manager.get_account(&id).await {
        Ok(Some(mut acc)) => {
            acc.label = payload.label;
            if let Err(e) = state.account_manager.upsert_account(acc).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            let _ = state.account_tx.send("updated".to_string());
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Account not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
#[derive(Deserialize)]
pub struct UpdateProxyStatusReq {
    pub disabled: bool,
}

pub async fn update_proxy_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProxyStatusReq>,
) -> impl IntoResponse {
    match state.account_manager.update_proxy_disabled(&id, payload.disabled).await {
        Ok(_) => {
            let _ = state.account_tx.send("updated".to_string());
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn switch_account_to_ide(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.account_manager.get_account(&id).await {
        Ok(Some(mut acc)) => {
            match transcoder_core::ide::switch_account(&acc).await {
                Ok(new_profile) => {
                    if let Some(profile) = new_profile {
                        acc.device_profile = Some(profile);
                        let _ = state.account_manager.upsert_account(acc).await;
                        let _ = state.account_tx.send("updated".to_string());
                    }
                    (StatusCode::OK, "Account switched to local IDE successfully").into_response()
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Account not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ReorderAccountsReq {
    pub ids: Vec<String>,
}

pub async fn reorder_accounts(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderAccountsReq>,
) -> impl IntoResponse {
    match state.account_manager.reorder_accounts(payload.ids).await {
        Ok(_) => {
            let _ = state.account_tx.send("reordered".to_string());
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
