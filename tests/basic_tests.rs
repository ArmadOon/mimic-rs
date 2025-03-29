use mimic_rs::MockServer;
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_basic_static_response() {
    let port = 9000;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/hello")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"message": "Hello, world!"}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let resp = client
        .get(format!("http://localhost:{}/api/hello", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["message"], "Hello, world!");

    assert_eq!(server.count_calls("GET", "/api/hello").await, 1);
}

#[tokio::test]
async fn test_path_not_found() {
    let port = 9001;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/defined")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"status": "ok"}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let resp = client
        .get(format!("http://localhost:{}/api/undefined", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_reset_server() {
    let port = 9002;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/test")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"test": true}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let resp = client
        .get(format!("http://localhost:{}/api/test", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    server.reset().await;

    let resp = client
        .get(format!("http://localhost:{}/api/test", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 404);
}
