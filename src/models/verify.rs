use serde::{Deserialize, Serialize};

/// Request for verifying the number of endpoint calls
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub method: String,

    pub path: String,

    pub times: usize,
}

/// Response for verifying the number of calls
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub path: String,

    pub method: String,

    pub expected: usize,

    pub actual: usize,

    pub success: bool,
}

impl VerifyResponse {
    /// Creates a new verification response
    pub fn new(method: String, path: String, expected: usize, actual: usize) -> Self {
        Self {
            method,
            path,
            expected,
            actual,
            success: expected == actual,
        }
    }
}
