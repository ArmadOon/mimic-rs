use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::response::MockResponse;

/// Represents an expectation that the server should fulfill
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MockExpectation {
    /// Unique identifier of the expectation
    pub id: String,

    /// HTTP Method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// Request path (may contain wildcard characters '*')
    pub path: String,

    /// Expected query parameters
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub query_params: HashMap<String, String>,

    /// Expected HTTP headers
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub headers: HashMap<String, String>,

    /// Expected request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Response that the server should return
    pub response: MockResponse,
}

impl MockExpectation {
    /// Creates a new expectation
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            method: method.to_uppercase(),
            path: path.to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            response: MockResponse::default(),
        }
    }
}

/// Represents a request to create an expectation
#[derive(Debug, Deserialize)]
pub struct CreateExpectationRequest {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// Request path (may contain wildcard characters '*')
    pub path: String,

    /// Expected query parameters
    #[serde(default)]
    pub query_params: HashMap<String, String>,

    /// Expected HTTP headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Expected request body
    pub body: Option<String>,

    /// Response that the server should return
    pub response: MockResponse,
}

impl From<CreateExpectationRequest> for MockExpectation {
    fn from(req: CreateExpectationRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            method: req.method,
            path: req.path,
            query_params: req.query_params,
            headers: req.headers,
            body: req.body,
            response: req.response,
        }
    }
}
