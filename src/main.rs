use axum::Router;
use webcaster_server::routes::livestream_route::create_livestream_route;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().nest("/livestream", create_livestream_route());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
