use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    response::IntoResponse,
};
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
    let query_string = req.uri().query();
    let headers = req.headers().clone();

    info!("Received request: {} {}", method, path);

    // Extract query params and headers
    let query_params = extract_query_params(query_string);
    let headers_map = extract_headers(&headers);

    // Now that we've extracted all needed data, we can consume req
    let (_, body) = req.into_parts();
    let body = extract_body_bytes(body).await;

    // Record the request
    server
        .record_request(
            method.to_string(),
            path.clone(),
            &query_params,
            &headers_map,
            body.as_deref(),
        )
        .await;

    let expectations = server.get_expectations_by_method(method.as_str()).await;
    if let Some(expectation) = find_matching_expectation(
        &expectations,
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
    match query {
        Some(q) if !q.is_empty() => {
            let capacity = q.matches('&').count() + 1;
            let mut params = HashMap::with_capacity(capacity);

            for pair in q.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                }
            }
            params
        }
        _ => HashMap::new(),
    }
}

/// Extracts HTTP headers
fn extract_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut result = HashMap::with_capacity(headers.len());

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

/// Finds matching expectation - simplified because we already filtered by method
fn find_matching_expectation(
    expectations: &[MockExpectation],
    path: &str,
    query_params: &HashMap<String, String>,
    headers: &HashMap<String, String>,
    body: Option<&str>,
) -> Option<MockExpectation> {
    for exp in expectations {
        // Check path (supports regex)
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

    // If we have a file and no cached content yet, load and cache it
    if let Some(file_name) = &expectation.response.body_file {
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
    }

    // Check if we have pre-serialized JSON content
    if let Some(json_str) = expectation.response.get_json_string() {
        return builder
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(json_str))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    // Fallback to other content types
    if let Some(content) = &expectation.response.cached_file_content {
        // Return as plain text
        return builder
            .body(axum::body::Body::from(content.clone()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    // Empty response
    builder
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
