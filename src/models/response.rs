use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents the response that the mock server returns when matching an expectation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MockResponse {
    /// HTTP status code of the response
    pub status_code: u16,

    /// HTTP headers of the response
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Body of the response as a JSON value (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,

    /// Path to the file with the response body (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_file: Option<String>,
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: None,
            body_file: None,
        }
    }
}

impl MockResponse {
    /// Creates a new response with the given status code
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            ..Default::default()
        }
    }

    /// Adds an HTTP header to the response
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the body of the response as a JSON value
    pub fn with_json_body(mut self, body: Value) -> Self {
        self.body = Some(body);

        // Adds the Content-Type header if it does not already exist
        if !self.headers.contains_key("Content-Type") {
            self.headers
                .insert("Content-Type".to_string(), "application/json".to_string());
        }

        self
    }

    /// Sets the body of the response as a file path
    pub fn with_json_file(mut self, file_path: &str) -> Self {
        self.body_file = Some(file_path.to_string());

        // Adds the Content-Type header if it does not already exist
        if !self.headers.contains_key("Content-Type") {
            self.headers
                .insert("Content-Type".to_string(), "application/json".to_string());
        }

        self
    }
}
