use serde_json::Value;

use super::MockServer;
use crate::models::MockExpectation;

/// Builder for defining expectations
pub struct ExpectationBuilder {
    /// Reference to the server
    server: MockServer,

    /// The expectation being built
    expectation: MockExpectation,
}

impl ExpectationBuilder {
    /// Creates a new builder for expectations
    pub(crate) fn new(server: MockServer) -> Self {
        Self {
            server,
            expectation: MockExpectation::new("GET", "/"),
        }
    }

    /// Sets the request path
    ///
    /// # Arguments
    /// * `path` - The request path (can contain wildcards '*')
    pub fn path(mut self, path: &str) -> Self {
        self.expectation.path = path.to_string();
        self
    }

    /// Sets the HTTP method
    ///
    /// # Arguments
    /// * `method` - The HTTP method (GET, POST, PUT, DELETE, etc.)
    pub fn method(mut self, method: &str) -> Self {
        self.expectation.method = method.to_uppercase();
        self
    }

    /// Adds an expected query parameter
    ///
    /// # Arguments
    /// * `key` - The parameter key
    /// * `value` - The parameter value
    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.expectation
            .query_params
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Adds an expected HTTP header
    ///
    /// # Arguments
    /// * `key` - The header key
    /// * `value` - The header value
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.expectation
            .headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the expected request body
    ///
    /// # Arguments
    /// * `body` - The expected request body
    pub fn body(mut self, body: &str) -> Self {
        self.expectation.body = Some(body.to_string());
        self
    }

    /// Starts defining the response
    pub fn respond(self) -> ResponseBuilder {
        ResponseBuilder::new(self)
    }
}

/// Builder for defining responses
pub struct ResponseBuilder {
    /// Reference to the expectation builder
    expectation_builder: ExpectationBuilder,
}

impl ResponseBuilder {
    /// Creates a new builder for responses
    fn new(expectation_builder: ExpectationBuilder) -> Self {
        Self {
            expectation_builder,
        }
    }

    /// Sets the HTTP status code of the response
    ///
    /// # Arguments
    /// * `status` - The HTTP status code
    pub fn status(mut self, status: u16) -> Self {
        self.expectation_builder.expectation.response.status_code = status;
        self
    }

    /// Adds an HTTP header to the response
    ///
    /// # Arguments
    /// * `key` - The header key
    /// * `value` - The header value
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.expectation_builder
            .expectation
            .response
            .headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the JSON body of the response
    ///
    /// # Arguments
    /// * `body` - The JSON value as the response body
    pub fn json(mut self, body: Value) -> Self {
        self.expectation_builder.expectation.response.body = Some(body);

        // Adds Content-Type header if it does not exist
        if !self
            .expectation_builder
            .expectation
            .response
            .headers
            .contains_key("Content-Type")
        {
            self.expectation_builder
                .expectation
                .response
                .headers
                .insert("Content-Type".to_string(), "application/json".to_string());
        }

        self
    }

    /// Sets the path to a JSON file as the response body
    ///
    /// # Arguments
    /// * `file_path` - The relative path to the JSON file in the resources directory
    pub fn json_file(mut self, file_path: &str) -> Self {
        self.expectation_builder.expectation.response.body_file = Some(file_path.to_string());

        // Adds Content-Type header if it does not exist
        if !self
            .expectation_builder
            .expectation
            .response
            .headers
            .contains_key("Content-Type")
        {
            self.expectation_builder
                .expectation
                .response
                .headers
                .insert("Content-Type".to_string(), "application/json".to_string());
        }

        self
    }

    /// Completes the expectation definition and adds it to the server
    pub fn build(self) {
        let server = self.expectation_builder.server.clone();
        let expectation = self.expectation_builder.expectation;

        // Use async blocks for operations with RwLock
        tokio::spawn(async move {
            server.add_expectation(expectation).await;
        });
    }
}
