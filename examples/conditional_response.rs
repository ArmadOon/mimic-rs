use chrono::Timelike;
use mimic_rs::{MockResponse, MockServer};
use rand::Rng;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicialization of the server
    let server = MockServer::new("./examples/resources");

    // Define expectations
    server
        .expect()
        .path("/api/rate-limited")
        .method("GET")
        .respond()
        .conditional(|count| {
            // first three calls return 200 OK
            if count <= 3 {
                MockResponse::new(200).with_json_body(json!({
                    "status": "success",
                    "calls": count,
                    "remaining": 3 - count
                }))
            } else {
                // fourth call returns 429 Too Many Requests
                MockResponse::new(429).with_json_body(json!({
                    "status": "error",
                    "message": "Rate limit exceeded",
                    "retry_after": 60
                }))
            }
        })
        .build()
        .await;

    // simulate a failing endpoint after 3 successful calls
    server
        .expect()
        .path("/api/failing")
        .method("GET")
        .respond()
        .conditional(|count| {
            if count < 3 {
                MockResponse::new(200).with_json_body(json!({"status": "ok"}))
            } else {
                MockResponse::new(500)
                    .with_json_body(json!({"status": "error", "message": "Internal server error"}))
            }
        })
        .build()
        .await;

    // Time-based responses (different responses based on time of day)
    server
        .expect()
        .path("/api/greeting")
        .method("GET")
        .respond()
        .conditional(|_| {
            let hour = chrono::Local::now().hour();

            if hour < 12 {
                MockResponse::new(200)
                    .with_json_body(json!({"greeting": "Good morning!", "hour": hour}))
            } else if hour < 18 {
                MockResponse::new(200)
                    .with_json_body(json!({"greeting": "Good afternoon!", "hour": hour}))
            } else {
                MockResponse::new(200)
                    .with_json_body(json!({"greeting": "Good evening!", "hour": hour}))
            }
        })
        .build()
        .await;

    server
        .expect()
        .path("/api/status")
        .method("GET")
        .respond()
        .conditional(|_| {
            // Randomly generate a response status
            let mut rng = rand::rng();
            let status_choice = rng.random_range(0..5);

            match status_choice {
                0 => {
                    // 200 OK d
                    MockResponse::new(200).with_json_body(json!({
                        "status": "success",
                        "message": "Request successful!",
                        "data": {
                            "items": [
                                { "id": 1, "name": "Item 1" },
                                { "id": 2, "name": "Item 2" },
                                { "id": 3, "name": "Item 3" }
                            ],
                            "total": 3
                        }
                    }))
                }
                1 => {
                    // 404 Not Found
                    MockResponse::new(404).with_json_body(json!({
                        "status": "error",
                        "error_code": "RESOURCE_NOT_FOUND",
                        "message": "Request resource not found",
                        "details": "Check the URL and try again"
                    }))
                }
                2 => {
                    // 403 Forbidden d
                    MockResponse::new(403).with_json_body(json!({
                        "status": "error",
                        "error_code": "ACCESS_DENIED",
                        "message": "Access to resource is denied",
                        "details": "You're not allowed to use this resource",
                    }))
                }
                3 => {
                    // 400 Bad Requestd
                    MockResponse::new(400).with_json_body(json!({
                        "status": "error",
                        "error_code": "INVALID_REQUEST",
                        "message": "Bad request",
                        "validation_errors": [
                            { "field": "email", "message": "Invalid email address" },
                            { "field": "age", "message": "Age must be over 18" }
                        ]
                    }))
                }
                _ => {
                    // 500 Internal Server Error d
                    MockResponse::new(500).with_json_body(json!({
                        "status": "error",
                        "error_code": "INTERNAL_ERROR",
                        "message": "Unexpected error occurred",
                        "request_id": "req-38f9d2e7-91a4-4978-83b7-e33ef9efee25"
                    }))
                }
            }
        })
        .build()
        .await;
    println!("Server running on http://localhost:8080");
    println!("Available endpoints:");
    println!("  GET /api/rate-limited - Returns 429 after 3 calls");
    println!("  GET /api/failing - Returns 500 after 2 successful calls");
    println!("  GET /api/greeting - Returns greeting based on time of day");
    println!("  GET /api/status - Returns random status code (200, 404, 403, 400, 500)");

    server.start(8080).await?;
    Ok(())
}
