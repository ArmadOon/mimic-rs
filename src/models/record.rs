use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a record of a request that the mock server received
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestRecord {
    /// HTTP method of the request
    pub method: String,

    /// Path of the request
    pub path: String,

    /// Query parameters of the request
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub query_params: HashMap<String, String>,

    /// HTTP headers of the request
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub headers: HashMap<String, String>,

    /// Body of the request (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Timestamp of the request
    pub timestamp: DateTime<Utc>,
}

impl RequestRecord {
    /// Creates a new request record
    pub fn new(
        method: String,
        path: String,
        query_params: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Self {
        Self {
            method,
            path,
            query_params,
            headers,
            body,
            timestamp: Utc::now(),
        }
    }
}
