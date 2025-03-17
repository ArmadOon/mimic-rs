use serde::{Deserialize, Serialize};

/// Request for verifying the number of endpoint calls
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// HTTP method
    pub method: String,

    /// Endpoint path
    pub path: String,

    /// Expected number of calls
    pub times: usize,
}

/// Response for verifying the number of calls
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    /// Endpoint path
    pub path: String,

    /// HTTP method
    pub method: String,

    /// Expected number of calls
    pub expected: usize,

    /// Actual number of calls
    pub actual: usize,

    /// Whether the verification succeeded (expected == actual)
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
