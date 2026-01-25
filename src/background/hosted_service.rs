
use std::sync::Arc;

use crate::background::job_queue::JobQueue;
use crate::di::ServiceProvider;

#[derive(Clone)]
pub struct HostedServiceContext {
    services: Arc<ServiceProvider>,
    job_queue: Option<Arc<dyn JobQueue>>,
}

impl HostedServiceContext {
    pub fn new(services: ServiceProvider) -> Self {
        Self {
            services: Arc::new(services),
            job_queue: None,
        }
    }

    pub fn with_job_queue(services: ServiceProvider, job_queue: Arc<dyn JobQueue>) -> Self {
        Self {
            services: Arc::new(services),
            job_queue: Some(job_queue),
        }
    }

    pub fn services(&self) -> &ServiceProvider {
        &self.services
    }

    pub fn job_queue(&self) -> Option<Arc<dyn JobQueue>> {
        self.job_queue.clone()
    }
}

pub trait HostedService: Send + Sync + 'static {
    fn start(&self, ctx: HostedServiceContext);
    fn stop(&self);
}

pub struct HostedServiceHost {
    services: Vec<Arc<dyn HostedService>>,
}

impl HostedServiceHost {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn add<S: HostedService + 'static>(&mut self, service: S) -> &mut Self {
        self.services.push(Arc::new(service));
        self
    }

    pub fn start(&self, ctx: HostedServiceContext) {
        for service in &self.services {
            service.start(ctx.clone());
        }
    }

    pub fn stop(&self) {
        for service in &self.services {
            service.stop();
        }
    }
}
