use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::server::MockServer;
/// Handler for resetting the server (clearing all expectations and records)
pub async fn handle_reset(State(server): State<MockServer>) -> impl IntoResponse {
    server.reset().await;

    StatusCode::OK
}
