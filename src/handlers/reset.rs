use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::server::MockServer;
/// Handler for resetting the server (clearing all expectations and records)
pub async fn handle_reset(State(server): State<MockServer>) -> impl IntoResponse {
    // Reset the server
    server.reset().await;

    // Return a successful response
    StatusCode::OK
}
