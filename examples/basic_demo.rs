use mimic_rs::MockServer;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mock server
    let server = MockServer::new("./examples/resources");

    // Define expectations - simple GET endpoint
    server
        .expect()
        .path("/api/hello")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({
          "message": "Hello, world!"
        }))
        .build()
        .await;

    // Define expectations - POST endpoint with body check
    server
        .expect()
        .path("/api/users")
        .method("POST")
        .body(r#"{"name":"John"}"#)
        .respond()
        .status(201)
        .json(json!({
          "id": 1,
          "name": "John",
          "created": true
        }))
        .build()
        .await;

    // Define expectations - GET endpoint with response from JSON file
    server
        .expect()
        .path("/api/users/42")
        .method("GET")
        .respond()
        .status(200)
        .json_file("user.json")
        .build()
        .await;

    println!("Server is running at http://localhost:8080");
    println!("Available endpoints:");
    println!("  GET  /api/hello");
    println!("  POST /api/users (with body {{'name':'John'}})");
    println!("  GET  /api/users/42");

    // Start the server
    server.start(8080).await?;

    Ok(())
}
