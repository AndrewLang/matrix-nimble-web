use std::sync::Arc;

use crate::app::builder::AppBuilder;
use crate::background::in_memory_queue::InMemoryJobQueue;
use crate::controller::controller::Controller;
use crate::di::ServiceContainer;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::pipeline::middleware::Middleware;

pub struct TestApp {
    builder: Option<AppBuilder>,
    background_queue: Option<Arc<InMemoryJobQueue>>,
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            builder: Some(AppBuilder::new()),
            background_queue: None,
        }
    }

    pub fn use_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        if let Some(builder) = self.builder.as_mut() {
            builder.use_middleware(middleware);
        }
        self
    }

    pub fn add_controller<T: Controller>(mut self) -> Self {
        if let Some(builder) = self.builder.as_mut() {
            builder.use_controller::<T>();
        }
        self
    }

    pub fn use_auth(mut self) -> Self {
        if let Some(builder) = self.builder.as_mut() {
            builder.use_authentication();
            builder.use_authorization();
        }
        self
    }

    pub fn run(self, request: HttpRequest) -> HttpResponse {
        let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
        rt.block_on(self.run_async(request))
    }

    pub async fn run_async(self, request: HttpRequest) -> HttpResponse {
        let mut builder = self.builder.expect("test app builder");
        if let Some(queue) = self.background_queue {
            builder.use_job_queue(queue.as_ref().clone());
        }
        let app = builder.build();
        app.handle_http_request(request).await
    }

    pub(crate) fn ensure_background_queue(&mut self) -> Arc<InMemoryJobQueue> {
        if let Some(queue) = self.background_queue.as_ref() {
            return queue.clone();
        }
        let services = Arc::new(ServiceContainer::new().build());
        let queue = Arc::new(InMemoryJobQueue::new(services));
        self.background_queue = Some(queue.clone());
        queue
    }
}
