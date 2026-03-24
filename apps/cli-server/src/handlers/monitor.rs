use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, Sse}},
    Json,
};
use std::sync::Arc;
use std::convert::Infallible;
use futures::Stream;
use serde::Deserialize;
use tokio_stream::StreamExt;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LogQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match state.traffic_mgr.get_recent_logs(limit, offset) {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn clear_logs(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.traffic_mgr.clear_all_logs() {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn monitor_stream(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut rx = state.traffic_mgr.broadcast_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(log) => {
                    if let Ok(data) = serde_json::to_string(&log) {
                        yield Ok::<Event, std::convert::Infallible>(Event::default().data(data));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(_) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

pub async fn system_logs_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.mem_logger.broadcast_tx.subscribe();
    let stream = async_stream::stream! {
        while let Ok(entry) = rx.recv().await {
            if let Ok(data) = serde_json::to_string(&entry) {
                yield Ok(Event::default().data(data));
            }
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}
