use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::response::MockResponse;

/// Represents an expectation that the server should fulfill
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MockExpectation {
    pub id: String,

    pub method: String,

    pub path: String,

    #[serde(skip)]
    pub path_regex: Option<Regex>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub query_params: HashMap<String, String>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub headers: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    pub response: MockResponse,
}

impl MockExpectation {
    /// Creates a new expectation
    pub fn new(method: &str, path: &str) -> Self {
        let mut exp = Self {
            id: Uuid::new_v4().to_string(),
            method: method.to_uppercase(),
            path: path.to_string(),
            path_regex: None,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            response: MockResponse::default(),
        };

        exp.compile_regex_if_needed();
        exp
    }

    /// Compiles the regex if the path contains wildcards
    pub fn compile_regex_if_needed(&mut self) {
        if self.path.contains('*') {
            let regex_path = self.path.replace('*', ".*");
            if let Ok(re) = Regex::new(&format!("^{}$", regex_path)) {
                self.path_regex = Some(re);
            }
        }
    }
}

/// Represents a request to create an expectation
#[derive(Debug, Deserialize)]
pub struct CreateExpectationRequest {
    pub method: String,

    pub path: String,

    #[serde(default)]
    pub query_params: HashMap<String, String>,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    pub body: Option<String>,

    pub response: MockResponse,
}

impl From<CreateExpectationRequest> for MockExpectation {
    fn from(req: CreateExpectationRequest) -> Self {
        let mut exp = Self {
            id: Uuid::new_v4().to_string(),
            method: req.method,
            path: req.path,
            path_regex: None, // Add the missing field
            query_params: req.query_params,
            headers: req.headers,
            body: req.body,
            response: req.response,
        };

        // Compile regex for paths with wildcards
        exp.compile_regex_if_needed();
        exp
    }
}
