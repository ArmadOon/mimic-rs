# mimic-rs

<div align="center">
  <img src="assets/mimic-rs-logo.svg" alt="mimic-rs logo" width="400" height="400">
</div>

![CI Status](https://github.com/ArmadOon/mimic-rs/workflows/Rust%20CI/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/mimic-rs.svg)](https://crates.io/crates/mimic-rs)
[![Documentation](https://docs.rs/mimic-rs/badge.svg)](https://docs.rs/mimic-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A flexible, high-performance HTTP mock server for testing HTTP integrations, written in Rust.

## Features

- Dynamic request matching based on HTTP method, path, query parameters, headers, and body
- Fluent API for defining expectations
- Response templating from JSON files
- Verification of invocations (count/patterns)
- Support for wildcard paths using regex patterns
- Thread-safe design for concurrent tests
- Fast startup time for integration into test suites
- Minimal dependencies

## Installation

Add mimic-rs to your `Cargo.toml`:

```toml
[dependencies]
mimic-rs = "0.1.0"
```

## Quick Start

```rust
use mimic_rs::MockServer;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mock server with path to response resources
    let server = MockServer::new("./resources");

    // Define expectations
    server.expect()
        .path("/api/users/42")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({
            "id": 42,
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .build();

    // Define expectation with response from file
    server.expect()
        .path("/api/products/*")  // Supports wildcards
        .method("GET")
        .respond()
        .status(200)
        .json_file("products.json")  // Load from ./resources/products.json
        .build();

    // Start the server
    println!("Mock server running on http://localhost:8080");
    server.start(8080).await?;

    Ok(())
}
```

## Using in Tests

```rust
use mimic_rs::MockServer;
use serde_json::json;
use reqwest::Client;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_my_api_client() {
    // Start server on a specific port for tests
    let port = 8090;
    let server = MockServer::new("./tests/resources");

    // Configure mock response
    server.expect()
        .path("/api/data")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"status": "success", "data": [1, 2, 3]}))
        .build();

    // Start server in background
    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    // Test your client
    let client = Client::new();
    let resp = client.get(&format!("http://localhost:{}/api/data", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    // Verify the endpoint was called
    assert_eq!(server.count_calls("GET", "/api/data").await, 1);
}
```

## Matching Requests

mimic-rs provides flexible request matching:

```rust
// Match by path with wildcard
server.expect()
    .path("/api/users/*/profile")
    .method("GET")
    .respond()
    .status(200)
    .json(json!({"name": "John"}))
    .build();

// Match with query parameters
server.expect()
    .path("/api/search")
    .method("GET")
    .query_param("q", "rust")
    .respond()
    .status(200)
    .json(json!({"results": ["Rust Programming"]}))
    .build();

// Match with headers
server.expect()
    .path("/api/protected")
    .method("GET")
    .header("Authorization", "Bearer token123")
    .respond()
    .status(200)
    .json(json!({"protected": true}))
    .build();

// Match with specific request body
server.expect()
    .path("/api/users")
    .method("POST")
    .body(r#"{"name":"Alice"}"#)
    .respond()
    .status(201)
    .json(json!({"id": 1, "name": "Alice"}))
    .build();
```

## HTTP API

mimic-rs provides an HTTP API that can be used by any HTTP client, making it framework and language agnostic:

```bash
# Set up an expectation
curl -X POST -H "Content-Type: application/json" -d '{
  "method": "GET",
  "path": "/api/users/1",
  "response": {
    "status_code": 200,
    "body": {"id": 1, "name": "John Doe"}
  }
}' http://localhost:8080/_setup

# Verify calls
curl -X POST -H "Content-Type: application/json" -d '{
  "method": "GET",
  "path": "/api/users/1",
  "times": 1
}' http://localhost:8080/_verify

# Reset the server
curl -X POST http://localhost:8080/_reset
```

## Java Integration (In Development)

Integration with Java testing frameworks is currently under development.

In the future, mimic-rs will provide a Java client that allows seamless integration with JUnit and other testing frameworks, making it easy to use this mock server in Java-based tests.

Currently, you can use mimic-rs with Java applications by:

1. Running the mock server as a separate process
2. Connecting to it via standard HTTP requests
3. Setting up expectations via the HTTP API endpoints

## API Documentation

### MockServer

The main interface for creating and managing the mock server.

```rust
// Create a new server
let server = MockServer::new("./resources");

// Start the server
server.start(8080).await?;

// Reset all expectations and logs
server.reset().await;

// Get count of calls to an endpoint
let count = server.count_calls("GET", "/api/users").await;
```

### ExpectationBuilder

Fluent API for defining request expectations.

```rust
server.expect()
    .path("/path")           // Set the request path
    .method("POST")          // Set the HTTP method
    .query_param("key", "value")  // Add query parameter
    .header("Content-Type", "application/json")  // Add header
    .body("{}")              // Set expected body
    .respond()               // Start defining response
    .status(201)             // Set response status
    .header("X-Custom", "value")  // Add response header
    .json(json!(...))        // Set JSON response body
    .json_file("file.json")  // Or load from file
    .build();                // Register the expectation
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
