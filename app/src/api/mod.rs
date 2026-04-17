use axum::{routing::get, Json, Router, serve};
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
struct MyResponse {
    message: String,
}

async fn hello() -> Json<MyResponse> {
    Json(MyResponse {
        message: "Hello, world!".into(),
    })
}

pub async fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!("HTTP server listening on {}", address);
    let app = Router::new()
        .route("/", get(hello));
    tokio::spawn(async move {
        let _ = serve(listener, app).await;
    });

    Ok(())
}