use axum::{routing::get, Router};

pub fn create_user_routes() -> Router {
    Router::new().route("/", get(|| async {}))
}