use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use std::sync::Arc;
use tokio_stream::StreamExt;
use futures::stream::Stream;
use std::convert::Infallible;
use crate::state::AppState;

/// GET /v1/accounts/events
/// 订阅账号变更事件流 (SSE)
pub async fn account_events_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.account_tx.subscribe();

    let stream = async_stream::stream! {
        // 连接建立时，先发送一个初始化的 ping
        yield Ok(Event::default().data("connected"));

        while let Ok(msg) = rx.recv().await {
            // 将变更类型（如 "refreshed", "imported", "deleted"）作为事件数据发送
            yield Ok(Event::default().data(msg));
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
