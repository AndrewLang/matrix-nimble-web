use std::sync::Arc;

use crate::di::ServiceProvider;

pub enum JobResult {
    Success,
    Failure(String),
}

#[derive(Clone)]
pub struct JobContext {
    services: Arc<ServiceProvider>,
}

impl JobContext {
    pub fn new(services: Arc<ServiceProvider>) -> Self {
        Self { services }
    }

    pub fn services(&self) -> &ServiceProvider {
        &self.services
    }
}

pub trait BackgroundJob: Send + Sync + 'static {
    fn execute(&self, ctx: JobContext) -> JobResult;
}
