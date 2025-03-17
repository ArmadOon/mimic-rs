mod dynamic;
mod reset;
mod setup;
mod verify;

use axum::{
    Router,
    routing::{any, post},
};
use tower_http::trace::TraceLayer;

use crate::server::MockServer;

/// Create a router for the server
pub fn create_router(server: MockServer) -> Router {
    // Create API router
    let api_router = Router::new()
        .route("/_setup", post(setup::handle_setup))
        .route("/_verify", post(verify::handle_verify))
        .route("/_reset", post(reset::handle_reset));

    // Create wildcard router for dynamic requests
    let dynamic_router = any(dynamic::handle_dynamic_request);

    // Combine routers
    Router::new()
        .merge(api_router)
        .fallback(dynamic_router)
        .layer(TraceLayer::new_for_http())
        .with_state(server)
}
