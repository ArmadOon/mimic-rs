use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::models::{VerifyRequest, VerifyResponse};
use crate::server::MockServer;

/// Handler for verifying the number of calls to an endpoint
pub async fn handle_verify(
    State(server): State<MockServer>,
    Json(request): Json<VerifyRequest>,
) -> impl IntoResponse {
    let actual = server.count_calls(&request.method, &request.path).await;

    let response = VerifyResponse::new(request.method, request.path, request.times, actual);

    if response.success {
        (StatusCode::OK, Json(response))
    } else {
        (StatusCode::BAD_REQUEST, Json(response))
    }
}
