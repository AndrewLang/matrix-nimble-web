use std::sync::Arc;

use crate::background::hosted_service::{HostedServiceContext, HostedServiceHost};
use crate::background::job_queue::JobQueue;
use crate::config::ConfigBuilder;
use crate::di::ServiceProvider;
use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::pipeline::pipeline::Pipeline;

pub struct Application {
    pipeline: Pipeline,
    services: ServiceProvider,
    hosted_services: HostedServiceHost,
    job_queue: Option<Arc<dyn JobQueue>>,
}

impl Application {
    pub(crate) fn new(
        pipeline: Pipeline,
        services: ServiceProvider,
        hosted_services: HostedServiceHost,
        job_queue: Option<Arc<dyn JobQueue>>,
    ) -> Self {
        Self {
            pipeline,
            services,
            hosted_services,
            job_queue,
        }
    }

    pub fn start(&self) {
        let ctx = match &self.job_queue {
            Some(queue) => {
                HostedServiceContext::with_job_queue(self.services.clone(), queue.clone())
            }
            None => HostedServiceContext::new(self.services.clone()),
        };
        self.hosted_services.start(ctx);
    }

    pub fn shutdown(&self) {
        self.hosted_services.stop();
    }

    pub fn services(&self) -> &ServiceProvider {
        &self.services
    }

    pub fn handle_http(&self, request: HttpRequest) -> HttpResponse {
        let services = self.services.clone();
        let config = ConfigBuilder::new().build();
        let mut context = HttpContext::new(request, services, config);
        let _ = self.pipeline.run(&mut context);
        context.response().clone()
    }
}
