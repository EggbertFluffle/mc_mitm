use axum::Router;
use tower_http::services::ServeDir;

pub async fn run(port: u16) {

    let app = Router::new().nest_service("/", ServeDir::new("src/frontend"));

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();
    println!("Http Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
