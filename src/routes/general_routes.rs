use crate::routes::utils::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

pub async fn handler() -> &'static str {
    "Hello, World!"
}

pub async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::from_u16(200).unwrap(), "Service Available")
}

pub async fn uptime(State(state): State<Arc<AppState>>) -> String {
    let uptime = state.start_time.elapsed();
    format!("Uptime: {} seconds", uptime.as_secs())
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
