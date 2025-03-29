use mimic_rs::MockResponse;
use mimic_rs::MockServer;
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_counter_based_conditional() {
    let port = 9010;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/counter")
        .method("GET")
        .respond()
        .conditional(|count| {
            if count <= 2 {
                MockResponse::new(200).with_json_body(json!({"count": count, "limit": false}))
            } else {
                MockResponse::new(429).with_json_body(json!({"count": count, "limit": true}))
            }
        })
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let url = format!("http://localhost:{}/api/counter", port);

    let resp1 = client.get(&url).send().await.unwrap();
    assert_eq!(resp1.status().as_u16(), 200);
    let body1: Value = resp1.json().await.unwrap();
    assert_eq!(body1["count"], 1);
    assert_eq!(body1["limit"], false);

    let resp2 = client.get(&url).send().await.unwrap();
    assert_eq!(resp2.status().as_u16(), 200);
    let body2: Value = resp2.json().await.unwrap();
    assert_eq!(body2["count"], 2);
    assert_eq!(body2["limit"], false);

    let resp3 = client.get(&url).send().await.unwrap();
    assert_eq!(resp3.status().as_u16(), 429);
    let body3: Value = resp3.json().await.unwrap();
    assert_eq!(body3["count"], 3);
    assert_eq!(body3["limit"], true);
}

#[tokio::test]
async fn test_status_code_based_conditionals() {
    let port = 9011;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/status-cycle")
        .method("GET")
        .respond()
        .conditional(|count| match count % 3 {
            0 => MockResponse::new(200).with_json_body(json!({"status": "success"})),
            1 => MockResponse::new(404).with_json_body(json!({"status": "not found"})),
            _ => MockResponse::new(500).with_json_body(json!({"status": "error"})),
        })
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let url = format!("http://localhost:{}/api/status-cycle", port);

    let resp1 = client.get(&url).send().await.unwrap();
    assert_eq!(resp1.status().as_u16(), 404);

    let resp2 = client.get(&url).send().await.unwrap();
    assert_eq!(resp2.status().as_u16(), 500);

    let resp3 = client.get(&url).send().await.unwrap();
    assert_eq!(resp3.status().as_u16(), 200);
}
