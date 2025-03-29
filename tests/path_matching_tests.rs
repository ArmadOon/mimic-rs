use mimic_rs::MockServer;
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_wildcard_paths() {
    let port = 9020;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/users/*/profile")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"profile": "data"}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    // Otestování různých cest, které by měly odpovídat vzoru
    let paths = vec![
        "/api/users/123/profile",
        "/api/users/abc/profile",
        "/api/users/user@example.com/profile",
    ];

    for path in paths {
        let resp = client
            .get(format!("http://localhost:{}{}", port, path))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status().as_u16(), 200);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["profile"], "data");
    }

    let resp = client
        .get(format!("http://localhost:{}/api/users/123/settings", port))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_multiple_wildcards() {
    let port = 9021;
    let server = MockServer::new("./tests/resources");

    server
        .expect()
        .path("/api/*/items/*")
        .method("GET")
        .respond()
        .status(200)
        .json(json!({"wildcards": true}))
        .build()
        .await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start(port).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();

    let paths = vec![
        "/api/users/items/123",
        "/api/products/items/abc",
        "/api/categories/items/xyz",
    ];

    for path in paths {
        let resp = client
            .get(format!("http://localhost:{}{}", port, path))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status().as_u16(), 200);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["wildcards"], true);
    }
}
