use crate::routes::utils::AppState;
use axum::{extract::ConnectInfo, extract::State, http::StatusCode, response::IntoResponse};
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> &'static str {
    println!("Received request from {}", addr);
    "Hello, World!"
}

pub async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::from_u16(200).unwrap(), "Service Available")
}

pub async fn uptime(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> String {
    println!("Received uptime request from {}", addr);
    let uptime = state.start_time.elapsed();
    format!("Uptime: {} seconds", uptime.as_secs())
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
