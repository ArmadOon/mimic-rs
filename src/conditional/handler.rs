use crate::models::MockResponse;
use std::sync::Arc;

/// Type of function for conditional responses
pub type ConditionalResponseFn = Arc<dyn Fn(usize) -> MockResponse + Send + Sync>;

/// Representation of a conditional response
#[derive(Clone)]
pub struct ConditionalResponse {
    pub handler: ConditionalResponseFn,

    pub call_count: usize,
}

impl ConditionalResponse {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(usize) -> MockResponse + Send + Sync + 'static,
    {
        Self {
            handler: Arc::new(handler),
            call_count: 0,
        }
    }

    pub fn generate_response(&mut self) -> MockResponse {
        self.call_count += 1;
        (self.handler)(self.call_count)
    }
}
