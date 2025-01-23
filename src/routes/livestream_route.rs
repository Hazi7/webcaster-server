use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::sync::{broadcast, RwLock};

use crate::{
    handlers::livestream_handler, services::wt_service::init_transport, AppState, GopCache,
};

pub fn create_livestream_route() -> Router {
    let (broadcaster, _) = broadcast::channel(100);

    // 初始化 AppState，包括 broadcaster 和 gop_cache
    let app_state = Arc::new(AppState {
        broadcaster,
        gop_cache: Arc::new(RwLock::new(GopCache::new())), // 10 MB 缓存
    });

    let app_state_clone = Arc::clone(&app_state);

    let axum_state_clone = Arc::clone(&app_state);

    tokio::spawn(async {
        let _ = init_transport(app_state_clone).await;
    });

    Router::new()
        .route("/push", get(livestream_handler::handle_ws_upgrade))
        .route("/pull", get(livestream_handler::handle_pull_client))
        .with_state(axum_state_clone)
}
