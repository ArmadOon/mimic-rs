use crate::models::MockExpectation;
use crate::models::MockResponse;
use crate::server::MockServer;
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
        return create_response(expectation, &server, server.resource_dir()).await;
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
    // Set a reasonable limit (10MB)
    const MAX_SIZE: usize = 10 * 1024 * 1024;

    match axum::body::to_bytes(body, MAX_SIZE).await {
        Ok(bytes) => {
            if bytes.is_empty() {
                None
            } else {
                // Convert bytes to string
                match String::from_utf8(bytes.to_vec()) {
                    Ok(body_string) => Some(body_string),
                    Err(e) => {
                        error!("Failed to convert request body to UTF-8: {}", e);
                        None
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read request body: {}", e);
            None
        }
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

/// Create response from mock
async fn create_response_from_mock(
    mut response: MockResponse,
    resource_dir: &FilePath,
) -> axum::response::Response {
    let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK);
    let mut builder = axum::response::Response::builder().status(status);

    for (key, value) in &response.headers {
        builder = builder.header(key, value);
    }

    if let Some(file_name) = &response.body_file {
        let file_path = resource_dir.join(file_name);
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                debug!("Loaded file {} for response", file_path.display());
                response.cache_file_content(content);
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

    if let Some(json_str) = response.get_json_string() {
        return builder
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(json_str))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    if let Some(content) = &response.cached_file_content {
        return builder
            .body(axum::body::Body::from(content.clone()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    builder
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Creates HTTP response based on expectation
async fn create_response(
    expectation: MockExpectation,
    server: &MockServer,
    resource_dir: &FilePath,
) -> axum::response::Response {
    if let Some(cond_id) = &expectation.response.conditional_id {
        let mut conditional_responses = server.conditional_responses.write().await;
        if let Some(conditional) = conditional_responses.get_mut(cond_id) {
            let response = conditional.generate_response();
            return create_response_from_mock(response, resource_dir).await;
        }
    }

    create_response_from_mock(expectation.response, resource_dir).await
}
