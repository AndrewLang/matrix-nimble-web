use std::sync::Arc;

use crate::app::application::Application;
use crate::background::hosted_service::{HostedService, HostedServiceHost};
use crate::background::job_queue::JobQueue;
use crate::controller::controller::Controller;
use crate::controller::registry::ControllerRegistry;
use crate::di::ServiceContainer;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::pipeline::Pipeline;
use crate::routing::router::Router;
use crate::routing::simple_router::SimpleRouter;
use crate::security::auth::AuthenticationMiddleware;
use crate::security::policy::AuthorizationMiddleware;
use crate::validation::ValidationMiddleware;

pub struct AppBuilder {
    pipeline: Pipeline,
    controller_registry: ControllerRegistry,
    router: SimpleRouter,
    services: ServiceContainer,
    hosted_services: HostedServiceHost,
    job_queue: Option<Arc<dyn JobQueue>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            controller_registry: ControllerRegistry::new(),
            router: SimpleRouter::new(),
            services: ServiceContainer::new(),
            hosted_services: HostedServiceHost::new(),
            job_queue: None,
        }
    }

    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.pipeline.add(middleware);
        self
    }

    pub fn add_controller<T: Controller>(&mut self) -> &mut Self {
        let mut registry = ControllerRegistry::new();
        T::register(&mut registry);

        self.controller_registry
            .merge_openapi_registry(registry.openapi_registry());

        for (route, endpoint) in registry
            .routes()
            .iter()
            .cloned()
            .zip(registry.endpoints().iter().cloned())
        {
            self.router.add_route(route.clone());
            self.controller_registry.add_route(route, endpoint);
        }

        self
    }

    pub fn use_authentication(&mut self) -> &mut Self {
        self.pipeline.add(AuthenticationMiddleware::new());
        self
    }

    pub fn use_authorization(&mut self) -> &mut Self {
        self.pipeline.add(AuthorizationMiddleware::new());
        self
    }

    pub fn use_validation(&mut self) -> &mut Self {
        self.pipeline.add(ValidationMiddleware::new());
        self
    }

    pub fn add_hosted_service<T: HostedService>(&mut self, service: T) -> &mut Self {
        self.hosted_services.add(service);
        self
    }

    pub fn add_job_queue<T>(&mut self, queue: T) -> &mut Self
    where
        T: JobQueue + 'static,
    {
        let queue = Arc::new(queue) as Arc<dyn JobQueue>;
        self.job_queue = Some(queue.clone());
        self.services
            .register_singleton::<Arc<dyn JobQueue>, _>(move |_| queue.clone());
        self
    }

    pub fn build(self) -> Application {
        let services = self.services.build();
        Application::new(self.pipeline, services, self.hosted_services, self.job_queue)
    }
}
