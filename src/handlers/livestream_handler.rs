use std::sync::Arc;

use axum::{
    body::Body,
    extract::{State, WebSocketUpgrade},
    http::Response,
    response::IntoResponse,
};

use crate::{services::livestream_service::Livestream, AppState};

pub async fn handle_ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| Livestream::handle_push_client(socket, state))
}

pub async fn handle_pull_client(state: State<Arc<AppState>>) -> Response<Body> {
    let base_timestamp = Some(0);
    let first_timestamp: Option<u32> = None;

    Response::builder()
        .header("Content-Type", "video/x-flv")
        .header("Connection", "keep-alive")
        .header("Cache-Control", "no-cache")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from_stream(
            Livestream::create_flv_stream(state, base_timestamp, first_timestamp).await,
        ))
        .unwrap()
}
