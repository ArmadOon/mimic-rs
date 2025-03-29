use mimic_rs::MockServer;
use reqwest::{Client, header};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_query_parameters() {
    let port = 9030;
    let server = MockServer::new("./tests/resources");

    // Define expectation with specific query parameters
    server
        .expect()
        .path("/api/search")
        .method("GET")
        .query_param("q", "test")
        .query_param("limit", "10")
        .respond()
        .status(200)
        .json(json!({"results": ["test1", "test2"]}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    // Correct request - with all required parameters
    let resp_ok = client
        .get(format!("http://localhost:{}/api/search", port))
        .query(&[("q", "test"), ("limit", "10")])
        .send()
        .await
        .unwrap();

    assert_eq!(resp_ok.status().as_u16(), 200);

    // Incorrect request - missing limit parameter
    let resp_wrong = client
        .get(format!("http://localhost:{}/api/search", port))
        .query(&[("q", "test")])
        .send()
        .await
        .unwrap();

    assert_eq!(resp_wrong.status().as_u16(), 404);

    // Incorrect request - different value for limit parameter
    let resp_wrong2 = client
        .get(format!("http://localhost:{}/api/search", port))
        .query(&[("q", "test"), ("limit", "20")])
        .send()
        .await
        .unwrap();

    assert_eq!(resp_wrong2.status().as_u16(), 404);
}

#[tokio::test]
async fn test_request_headers() {
    let port = 9031;
    let server = MockServer::new("./tests/resources");

    // Define expectation with required headers
    server
        .expect()
        .path("/api/secure")
        .method("GET")
        .header("Authorization", "Bearer token123")
        .header("X-API-Key", "secret-key")
        .respond()
        .status(200)
        .json(json!({"authorized": true}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    // Correct request - with all required headers
    let resp_ok = client
        .get(format!("http://localhost:{}/api/secure", port))
        .header("Authorization", "Bearer token123")
        .header("X-API-Key", "secret-key")
        .send()
        .await
        .unwrap();

    assert_eq!(resp_ok.status().as_u16(), 200);

    // Incorrect request - missing one header
    let resp_wrong = client
        .get(format!("http://localhost:{}/api/secure", port))
        .header("Authorization", "Bearer token123")
        .send()
        .await
        .unwrap();

    assert_eq!(resp_wrong.status().as_u16(), 404);

    // Incorrect request - wrong value for one header
    let resp_wrong2 = client
        .get(format!("http://localhost:{}/api/secure", port))
        .header("Authorization", "Bearer wrong-token")
        .header("X-API-Key", "secret-key")
        .send()
        .await
        .unwrap();

    assert_eq!(resp_wrong2.status().as_u16(), 404);
}

#[tokio::test]
async fn test_request_body() {
    // Set up the server
    let port = 9032;
    let server = MockServer::new("./tests/resources");

    // Define expectation with required body
    server
        .expect()
        .path("/api/echo")
        .method("POST")
        .body(r#"{"message":"hello"}"#)
        .respond()
        .status(200)
        .json(json!({"echoed": true, "original": {"message":"hello"}}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    // Correct request - with correct body
    let resp_ok = client
        .post(format!("http://localhost:{}/api/echo", port))
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"message":"hello"}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp_ok.status().as_u16(), 200);

    // Incorrect request - different body
    let resp_wrong = client
        .post(format!("http://localhost:{}/api/echo", port))
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"message":"different"}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp_wrong.status().as_u16(), 404);
}
