use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents the response that the mock server returns when matching an expectation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MockResponse {
    pub status_code: u16,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_file: Option<String>,

    #[serde(skip)]
    pub cached_file_content: Option<String>,

    #[serde(skip)]
    pub cached_json_content: Option<Value>,

    #[serde(skip)]
    pub conditional_id: Option<String>,
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: None,
            body_file: None,
            cached_file_content: None,
            cached_json_content: None,
            conditional_id: None,
        }
    }
}

impl MockResponse {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            ..Default::default()
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_json_body(mut self, body: Value) -> Self {
        self.body = Some(body);

        if !self.headers.contains_key("Content-Type") {
            self.headers
                .insert("Content-Type".to_string(), "application/json".to_string());
        }

        self
    }

    /// Sets the body of the response as a file path
    pub fn with_json_file(mut self, file_path: &str) -> Self {
        self.body_file = Some(file_path.to_string());

        if !self.headers.contains_key("Content-Type") {
            self.headers
                .insert("Content-Type".to_string(), "application/json".to_string());
        }

        self
    }

    /// Cache the content of the file to avoid repeated disk reads
    pub fn cache_file_content(&mut self, content: String) {
        // Try to parse as JSON first
        if let Ok(json_value) = serde_json::from_str::<Value>(&content) {
            self.cached_json_content = Some(json_value);
        }

        self.cached_file_content = Some(content);
    }

    /// Get the pre-serialized JSON string if available
    pub fn get_json_string(&self) -> Option<String> {
        match (&self.body, &self.cached_json_content) {
            (Some(body), _) => serde_json::to_string(body).ok(),
            (_, Some(json)) => serde_json::to_string(json).ok(),
            _ => None,
        }
    }

    pub fn with_conditional_id(mut self, id: String) -> Self {
        self.conditional_id = Some(id);
        self
    }
}
