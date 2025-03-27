use mimic_rs::{MockResponse, MockServer};
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

    println!("Server running on http://localhost:8080");
    println!("Available endpoints:");
    println!("  GET /api/rate-limited - Returns 429 after 3 calls");
    println!("  GET /api/failing - Returns 500 after 2 successful calls");

    server.start(8080).await?;
    Ok(())
}
