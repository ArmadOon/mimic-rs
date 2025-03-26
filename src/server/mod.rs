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
    expectations: Arc<RwLock<HashMap<String, Vec<MockExpectation>>>>,

    request_log: Arc<RwLock<Vec<RequestRecord>>>,

    resource_dir: PathBuf,

    max_request_log_size: usize,
}

impl MockServer {
    pub fn new<P: Into<PathBuf>>(resource_dir: P) -> Self {
        Self {
            expectations: Arc::new(RwLock::new(HashMap::new())),
            request_log: Arc::new(RwLock::new(Vec::new())),
            resource_dir: resource_dir.into(),
            max_request_log_size: 1000,
        }
    }
    /// Sets the maximum size of the request log
    pub fn with_max_log_size(mut self, size: usize) -> Self {
        self.max_request_log_size = size;
        self
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
    ///     .build().await;
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
    ///     .build().await;
    ///
    /// // Start the server
    /// server.start(8080).await.unwrap();
    /// # }
    /// ```
    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Preload file content before starting
        self.preload_file_content().await;

        let app = self.create_router();

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!("MockServer running at http://{}", addr);

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
    pub(crate) async fn add_expectation(&self, mut expectation: MockExpectation) {
        // Ensure the regex is compiled if needed
        expectation.compile_regex_if_needed();

        let mut expectations = self.expectations.write().await;

        expectations
            .entry(expectation.method.clone())
            .or_insert_with(Vec::new)
            .push(expectation);
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

        // Trim log if it exceeds the maximum size
        if request_log.len() > self.max_request_log_size {
            let to_remove = request_log.len() - self.max_request_log_size;
            request_log.drain(0..to_remove);
        }
    }

    /// Clears only the request log without affecting expectations
    pub async fn clear_request_log(&self) {
        let mut request_log = self.request_log.write().await;
        request_log.clear();
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
        expectations
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }

    /// Get expectations for a specific method (performance optimization)
    pub async fn get_expectations_by_method(&self, method: &str) -> Vec<MockExpectation> {
        let expectations = self.expectations.read().await;
        match expectations.get(method) {
            Some(exps) => exps.clone(),
            None => Vec::new(),
        }
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

    /// Preloads content from response files to avoid repeated disk reads
    pub async fn preload_file_content(&self) {
        use std::fs;
        use tracing::error;

        let resource_dir = self.resource_dir.clone();
        let mut expectations = self.expectations.write().await;

        for exps in expectations.values_mut() {
            for exp in exps.iter_mut() {
                if let Some(file_name) = &exp.response.body_file {
                    if exp.response.cached_file_content.is_none() {
                        let file_path = resource_dir.join(file_name);
                        match fs::read_to_string(&file_path) {
                            Ok(content) => {
                                info!("Preloaded file {} for response", file_path.display());
                                exp.response.cache_file_content(content);
                            }
                            Err(e) => {
                                error!("Error reading file {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }
            }
        }
    }
}
