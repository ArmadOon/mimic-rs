use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode},
    response::IntoResponse,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path as FilePath;
use tracing::{debug, error, info};

use crate::models::MockExpectation;
use crate::server::MockServer;

/// Handler for processing dynamic requests
pub async fn handle_dynamic_request(
    State(server): State<MockServer>,
    req: Request<Body>,
) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query_string = req.uri().query().map(|q| q.to_string());
    let headers_map = extract_headers(req.headers());

    info!("Received request: {} {}", method, path);

    let query_params = extract_query_params(query_string.as_deref());

    let (_, body) = req.into_parts();
    let body = extract_body_bytes(body).await;

    server
        .record_request(
            method.to_string(), 
            path.to_string(),   
            &query_params,
            &headers_map,
            body.as_deref(),
        )
        .await;

    let expectations = server.get_expectations().await;
    if let Some(expectation) = find_matching_expectation(
        &expectations,
        &method,          
        &path,             
        &query_params,
        &headers_map,      
        body.as_deref(),
    ) {
        return create_response(expectation, server.resource_dir()).await;
    }

    // If no matching expectation is found, return 404
    (
        StatusCode::NOT_FOUND,
        format!("No matching expectation found for {} {}", method, path),
    )
        .into_response()
}

/// Extracts query parameters from URL
fn extract_query_params(query: Option<&str>) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(key.to_string(), value.to_string());
            }
        }
    }

    params
}

/// Extracts HTTP headers
fn extract_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            result.insert(name.to_string(), value_str.to_string());
        }
    }

    result
}

/// Extracts request body from body parts
async fn extract_body_bytes(body: Body) -> Option<String> {
    match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => {
            if bytes.is_empty() {
                None
            } else {
                // Convert bytes to string
                match String::from_utf8(bytes.to_vec()) {
                    Ok(body_string) => Some(body_string),
                    Err(_) => None,
                }
            }
        }
        Err(_) => None,
    }
}

/// Finds matching expectation
fn find_matching_expectation(
    expectations: &[MockExpectation],
    method: &Method,              
    path: &str,                   
    query_params: &HashMap<String, String>,
    headers: &HashMap<String, String>, 
    body: Option<&str>,
) -> Option<MockExpectation> {
    for exp in expectations {
        if exp.method != method.as_str() {
            continue;
        }

        let path_matches = if let Some(regex) = &exp.path_regex {
            regex.is_match(path)
        } else {
            exp.path == path
        };

        if !path_matches {
            continue;
        }

        let mut query_params_match = true;
        for (key, value) in &exp.query_params {
            if query_params.get(key) != Some(value) {
                query_params_match = false;
                break;
            }
        }
        if !query_params_match {
            continue;
        }

        let mut headers_match = true;
        for (key, value) in &exp.headers {
            if headers.get(key) != Some(value) {
                headers_match = false;
                break;
            }
        }
        if !headers_match {
            continue;
        }

        if let Some(exp_body) = &exp.body {
            if body != Some(exp_body.as_str()) {
                continue;
            }
        }

        return Some(exp.clone());
    }

    // No matching expectation found
    None
}

/// Creates HTTP response based on expectation
async fn create_response(
    mut expectation: MockExpectation,
    resource_dir: &FilePath,
) -> axum::response::Response {
    // Create response builder
    let status = StatusCode::from_u16(expectation.response.status_code).unwrap_or(StatusCode::OK);
    let mut builder = axum::response::Response::builder().status(status);

    // Add headers
    for (key, value) in &expectation.response.headers {
        builder = builder.header(key, value);
    }

    // Return either file or JSON body
    if let Some(file_name) = &expectation.response.body_file {
        // Check if we have already cached the file content
        if expectation.response.cached_file_content.is_none() {
            let file_path = resource_dir.join(file_name);
            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    debug!("Loaded file {} for response", file_path.display());
                    expectation.response.cache_file_content(content);
                }
                Err(e) => {
                    error!("Error reading file {}: {}", file_path.display(), e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Error reading file: {}", e),
                    )
                        .into_response();
                }
            }
        }

        if let Some(content) = &expectation.response.cached_file_content {
            // Try to parse as JSON
            match serde_json::from_str::<Value>(content) {
                Ok(json_value) => {
                    return builder
                        .header("Content-Type", "application/json")
                        .body(axum::body::Body::from(
                            serde_json::to_string(&json_value).unwrap_or_else(|_| "{}".to_string()),
                        ))
                        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                }
                Err(_) => {
                    // Return as plain text
                    return builder
                        .body(axum::body::Body::from(content.clone()))
                        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                }
            }
        }
    } else if let Some(body) = &expectation.response.body {
        return builder
            .body(axum::body::Body::from(
                serde_json::to_string(body).unwrap_or_else(|_| "{}".to_string()),
            ))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    // Empty response
    builder
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
