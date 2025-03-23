use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::models::{CreateExpectationRequest, MockExpectation};
use crate::server::MockServer;

/// Handler for setting up a new expectation
pub async fn handle_setup(
    State(server): State<MockServer>,
    Json(request): Json<CreateExpectationRequest>,
) -> impl IntoResponse {
    let expectation: MockExpectation = request.into();

    server.add_expectation(expectation.clone()).await;

    (StatusCode::CREATED, Json(expectation))
}
