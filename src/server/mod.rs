pub mod expectation_builder;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tokio::sync::RwLock;
use tracing::info;

use self::expectation_builder::ExpectationBuilder;
use crate::handlers;
use crate::models::{MockExpectation, RequestRecord};

/// Main structure of the MockServer
#[derive(Clone)]
pub struct MockServer {
    expectations: Arc<RwLock<Vec<MockExpectation>>>,

    request_log: Arc<RwLock<Vec<RequestRecord>>>,

    resource_dir: PathBuf,
}

impl MockServer {
    pub fn new<P: Into<PathBuf>>(resource_dir: P) -> Self {
        Self {
            expectations: Arc::new(RwLock::new(Vec::new())),
            request_log: Arc::new(RwLock::new(Vec::new())),
            resource_dir: resource_dir.into(),
        }
    }

    /// Starts defining an expectation for a path
    ///
    /// # Arguments
    /// * `path` - Request path (can contain wildcards '*')
    ///
    /// # Example
    /// ```
    /// # use mimic_rs::MockServer;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// let server = MockServer::new("./resources");
    ///
    /// server.expect()
    ///     .path("/api/users/1")
    ///     .method("GET")
    ///     .respond()
    ///     .status(200)
    ///     .json_file("user.json")
    ///     .build();
    /// # }
    /// ```
    pub fn expect(&self) -> ExpectationBuilder {
        ExpectationBuilder::new(self.clone())
    }

    /// Starts the server on the specified port
    ///
    /// # Example
    /// ```no_run
    ///
    /// # use mimic_rs::MockServer;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// let server = MockServer::new("./resources");
    ///
    /// // Define expectations
    /// server.expect()
    ///     .path("/api/users/1")
    ///     .method("GET")
    ///     .respond()
    ///     .status(200)
    ///     .json_file("user.json")
    ///     .build();
    ///
    /// // Start the server
    /// server.start(8080).await.unwrap();
    /// # }
    /// ```
    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_router();

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!("MockServer running at http://{}", addr);

        // For Axum 0.8
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Creates a router for the server
    fn create_router(&self) -> Router {
        handlers::create_router(self.clone())
    }

    /// Adds an expectation to the server
    ///
    /// This method is primarily used by `ExpectationBuilder::build`
    pub(crate) async fn add_expectation(&self, expectation: MockExpectation) {
        let mut expectations = self.expectations.write().await;
        expectations.push(expectation);
    }

    pub(crate) async fn record_request(
        &self,
        method: String,
        path: String,
        query_params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body: Option<&str>,
    ) {
        let record = RequestRecord::new(
            method,
            path,
            query_params.clone(),
            headers.clone(),
            body.map(String::from),
        );
        let mut request_log = self.request_log.write().await;
        request_log.push(record);
    }

    pub async fn reset(&self) {
        {
            let mut expectations = self.expectations.write().await;
            expectations.clear();
        }

        {
            let mut request_log = self.request_log.write().await;
            request_log.clear();
        }
    }

    pub async fn get_expectations(&self) -> Vec<MockExpectation> {
        let expectations = self.expectations.read().await;
        expectations.clone()
    }

    pub async fn get_request_log(&self) -> Vec<RequestRecord> {
        let request_log = self.request_log.read().await;
        request_log.clone()
    }

    pub async fn count_calls(&self, method: &str, path: &str) -> usize {
        let request_log = self.request_log.read().await;
        request_log
            .iter()
            .filter(|r| r.method == method && r.path == path)
            .count()
    }

    pub fn resource_dir(&self) -> &PathBuf {
        &self.resource_dir
    }
}
