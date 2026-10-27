use std::sync::{Arc, RwLock};

use axum::{
    extract::State,
    routing::get,
    Json,
    Router
};
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use crate::mc::state::GameState;

pub async fn run(port: u16, shared_state: Arc<RwLock<GameState>>) {
    let app = Router::new()
        .route("/api/game_state", get(game_state))
        .route("/api/health", get(health_check))
        .nest_service("/", ServeDir::new("src/frontend"))
        .with_state(shared_state);

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();
    println!("Http Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn game_state(
    State(game_state): State<Arc<RwLock<GameState>>>,
) -> Json<GameState> {
    let snapshot = {
        let state = game_state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.clone()
    };

    Json(snapshot)
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
