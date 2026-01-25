use std::sync::Arc;

use crate::app::application::Application;
use crate::background::hosted_service::{HostedService, HostedServiceHost};
use crate::background::job_queue::JobQueue;
use crate::controller::controller::Controller;
use crate::controller::registry::ControllerRegistry;
use crate::di::ServiceContainer;
use crate::entity::entity::Entity;
use crate::entity::registry::EntityRegistry;
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
    entity_registry: EntityRegistry,
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
            entity_registry: EntityRegistry::new(),
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

    pub fn add_entity<T: Entity>(&mut self) -> &mut Self {
        self.entity_registry.register::<T>();
        self.controller_registry.register_entity::<T>();
        self
    }

    pub fn build(self) -> Application {
        let AppBuilder {
            pipeline,
            controller_registry: _,
            router: _,
            mut services,
            hosted_services,
            job_queue,
            entity_registry,
        } = self;
        let registry = Arc::new(entity_registry);
        services.register_singleton::<Arc<EntityRegistry>, _>(move |_| registry.clone());
        let services = services.build();
        Application::new(pipeline, services, hosted_services, job_queue)
    }

    pub(crate) fn entity_registry_clone(&self) -> EntityRegistry {
        EntityRegistry::from_registry(&self.entity_registry)
    }
}
