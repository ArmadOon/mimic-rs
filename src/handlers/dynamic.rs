use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode},
    response::IntoResponse,
};
use regex::Regex;
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
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let headers = req.headers().clone();

    info!("Received request: {} {}", method, path);

    // Extract query parameters
    let query_params = extract_query_params(uri.query());

    // Extract request body
    let body = extract_body(req).await;

    // Record the request
    let headers_map = extract_headers(&headers);
    server
        .record_request(
            method.to_string(),
            path.clone(),
            query_params.clone(),
            headers_map.clone(),
            body.clone(),
        )
        .await;

    // Find matching expectation
    let expectations = server.get_expectations().await;
    if let Some(expectation) = find_matching_expectation(
        &expectations,
        &method,
        &path,
        &query_params,
        &headers,
        body.as_deref(),
    ) {
        // Create response
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

/// Extracts request body and additional request information
async fn extract_body(req: Request<Body>) -> Option<String> {
    // Extract parts of the request
    let (parts, body) = req.into_parts();

    // Log additional request information if needed
    debug!(
        "Processing request: {} {} (version: {:?})",
        parts.method, parts.uri, parts.version
    );

    // You could also access:
    // - parts.extensions
    // - parts.uri.query()
    // - parts.uri.host()
    // - parts.uri.scheme()

    // Read body as bytes
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
    headers: &HeaderMap,
    body: Option<&str>,
) -> Option<MockExpectation> {
    for exp in expectations {
        // Check method
        if exp.method != method.as_str() {
            continue;
        }

        // Check path (supports regex)
        if exp.path.contains('*') {
            // Convert path with wildcards to regex
            let regex_path = exp.path.replace('*', ".*");
            if let Ok(re) = Regex::new(&format!("^{}$", regex_path)) {
                if !re.is_match(path) {
                    continue;
                }
            } else {
                // Skip this expectation on regex error
                continue;
            }
        } else if exp.path != path {
            continue;
        }

        // Check query parameters
        let mut query_params_match = true;
        for (key, value) in &exp.query_params {
            if let Some(param_value) = query_params.get(key) {
                if param_value != value {
                    query_params_match = false;
                    break;
                }
            } else {
                query_params_match = false;
                break;
            }
        }
        if !query_params_match {
            continue;
        }

        // Check headers
        let mut headers_match = true;
        for (key, value) in &exp.headers {
            if let Some(header_value) = headers.get(key) {
                if let Ok(header_str) = header_value.to_str() {
                    if header_str != value {
                        headers_match = false;
                        break;
                    }
                } else {
                    headers_match = false;
                    break;
                }
            } else {
                headers_match = false;
                break;
            }
        }
        if !headers_match {
            continue;
        }

        // Check body
        if let Some(exp_body) = &exp.body {
            if let Some(req_body) = body {
                if exp_body != req_body {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Return a copy of the matching expectation
        return Some(exp.clone());
    }

    // No matching expectation found
    None
}

/// Creates HTTP response based on expectation
async fn create_response(
    expectation: MockExpectation,
    resource_dir: &FilePath,
) -> axum::response::Response {
    // Create response builder
    let status = StatusCode::from_u16(expectation.response.status_code).unwrap_or(StatusCode::OK);

    let mut builder = axum::response::Response::builder().status(status);

    // Add headers
    for (key, value) in expectation.response.headers {
        builder = builder.header(&key, &value);
    }

    // Return either file or JSON body
    if let Some(file_name) = expectation.response.body_file {
        let file_path = resource_dir.join(&file_name);
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                debug!("Loaded file {} for response", file_path.display());

                // Try to parse as JSON
                match serde_json::from_str::<Value>(&content) {
                    Ok(json_value) => {
                        return builder
                            .header("Content-Type", "application/json")
                            .body(axum::body::Body::from(
                                serde_json::to_string(&json_value)
                                    .unwrap_or_else(|_| "{}".to_string()),
                            ))
                            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                    }
                    Err(_) => {
                        // Return as plain text
                        return builder
                            .body(axum::body::Body::from(content))
                            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                    }
                }
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
    } else if let Some(body) = expectation.response.body {
        return builder
            .body(axum::body::Body::from(
                serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string()),
            ))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    // Empty response
    builder
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
