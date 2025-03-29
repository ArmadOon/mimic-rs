use mimic_rs::MockServer;
use reqwest::{Client, header};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_setup_api() {
    let port = 9040;
    let server = MockServer::new("./tests/resources");

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    let setup_resp = client
        .post(format!("http://localhost:{}/_setup", port))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&json!({
            "method": "GET",
            "path": "/api/dynamic",
            "response": {
                "status_code": 200,
                "body": {"message": "Created via API"}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(setup_resp.status().as_u16(), 201);

    let resp = client
        .get(format!("http://localhost:{}/api/dynamic", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["message"], "Created via API");
}

#[tokio::test]
async fn test_verify_api() {
    let port = 9041;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/verification-test")
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

    for _ in 0..3 {
        client
            .get(format!("http://localhost:{}/api/verification-test", port))
            .send()
            .await
            .unwrap();
    }

    let verify_resp = client
        .post(format!("http://localhost:{}/_verify", port))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&json!({
            "method": "GET",
            "path": "/api/verification-test",
            "times": 3
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(verify_resp.status().as_u16(), 200);

    let verify_wrong = client
        .post(format!("http://localhost:{}/_verify", port))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&json!({
            "method": "GET",
            "path": "/api/verification-test",
            "times": 5
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(verify_wrong.status().as_u16(), 400);
}

#[tokio::test]
async fn test_reset_api() {
    let port = 9042;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/reset-test")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"before_reset": true}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    let resp_before = client
        .get(format!("http://localhost:{}/api/reset-test", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp_before.status().as_u16(), 200);

    let reset_resp = client
        .post(format!("http://localhost:{}/_reset", port))
        .send()
        .await
        .unwrap();

    assert_eq!(reset_resp.status().as_u16(), 200);

    let resp_after = client
        .get(format!("http://localhost:{}/api/reset-test", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp_after.status().as_u16(), 404);
}
