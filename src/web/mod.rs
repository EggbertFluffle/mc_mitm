use axum::{
    routing::{get, post},
    Json,
    Router
};
use serde_json::{json, Value};
use tower_http::services::ServeDir;

pub async fn run(port: u16) {
    let app = Router::new()
        .route("/api/health", get(health_check))
        .nest_service("/", ServeDir::new("src/frontend"));

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();
    println!("Http Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

async fn get_server_info() -> Json<Value> {
    Json(json!({
        // MITM Target IP
        // Server IP
    }))
}
