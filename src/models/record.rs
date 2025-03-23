use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a record of a request that the mock server received
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestRecord {
    pub method: String,

    pub path: String,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub query_params: HashMap<String, String>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub headers: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

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
