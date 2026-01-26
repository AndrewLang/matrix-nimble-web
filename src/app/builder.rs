use std::path::Path;
use std::sync::Arc;

use crate::app::application::Application;
use crate::background::hosted_service::{HostedService, HostedServiceHost};
use crate::background::in_memory_queue::InMemoryJobQueue;
use crate::background::job_queue::JobQueue;
use crate::config::ConfigBuilder;
use crate::controller::controller::Controller;
use crate::controller::registry::ControllerRegistry;
use crate::di::ServiceContainer;
use crate::entity::entity::Entity;
use crate::entity::registry::EntityRegistry;
use crate::middleware::endpoint_exec::EndpointExecutionMiddleware;
use crate::middleware::routing::RoutingMiddleware;
use crate::pipeline::middleware::{DynMiddleware, Middleware};
use crate::pipeline::pipeline::Pipeline;
use crate::routing::default_router::DefaultRouter;
use crate::routing::router::Router;
use crate::security::auth::AuthenticationMiddleware;
use crate::security::policy::AuthorizationMiddleware;
use crate::validation::ValidationMiddleware;

pub struct AppBuilder {
    pipeline: Pipeline,
    controller_registry: ControllerRegistry,
    router: DefaultRouter,
    services: ServiceContainer,
    hosted_services: HostedServiceHost,
    job_queue: JobQueueConfig,
    entity_registry: EntityRegistry,
    address: Option<String>,
    config_builder: ConfigBuilder,
}

enum JobQueueConfig {
    None,
    Provided(Arc<dyn JobQueue>),
    InMemory,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            controller_registry: ControllerRegistry::new(),
            router: DefaultRouter::new(),
            services: ServiceContainer::new(),
            hosted_services: HostedServiceHost::new(),
            job_queue: JobQueueConfig::None,
            entity_registry: EntityRegistry::new(),
            address: None,
            config_builder: ConfigBuilder::new(),
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
        self.job_queue = JobQueueConfig::Provided(queue.clone());
        self.services
            .register_singleton::<Arc<dyn JobQueue>, _>(move |_| queue.clone());
        self
    }

    pub fn use_in_memory_job_queue(&mut self) -> &mut Self {
        self.job_queue = JobQueueConfig::InMemory;
        self.services
            .register_singleton::<Arc<dyn JobQueue>, _>(move |provider| {
                Arc::new(InMemoryJobQueue::new(Arc::new(provider.clone())))
            });
        self
    }

    pub fn add_entity<T: Entity>(&mut self) -> &mut Self {
        self.entity_registry.register::<T>();
        self.controller_registry.register_entity::<T>();
        self
    }

    pub fn use_address(&mut self, address: &str) -> &mut Self {
        self.address = Some(address.to_string());
        self
    }

    pub fn use_address_env(&mut self, env_name: &str) -> &mut Self {
        let address = std::env::var(env_name).unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        self.address = Some(address);
        self
    }

    pub fn use_address_env_or(&mut self, env_name: &str, default: &str) -> &mut Self {
        let address = std::env::var(env_name).unwrap_or_else(|_| default.to_string());
        self.address = Some(address);
        self
    }

    pub fn use_config<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        let path = path.as_ref().to_path_buf();
        let path = if path.is_absolute() {
            path
        } else {
            let base = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|parent| parent.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            base.join(path)
        };
        let builder = std::mem::replace(&mut self.config_builder, ConfigBuilder::new());
        self.config_builder = builder.with_file(&path);
        self
    }

    pub fn use_env(&mut self) -> &mut Self {
        self.use_env_prefix("NIMBLE_")
    }

    pub fn use_env_prefix(&mut self, prefix: &str) -> &mut Self {
        let builder = std::mem::replace(&mut self.config_builder, ConfigBuilder::new());
        self.config_builder = builder.with_env(prefix);
        self
    }

    pub fn build(self) -> Application {
        let AppBuilder {
            pipeline,
            mut controller_registry,
            mut router,
            mut services,
            hosted_services,
            job_queue,
            entity_registry,
            address,
            config_builder,
        } = self;

        if controller_registry.ensure_openapi_endpoint() {
            router.add_route(crate::routing::route::Route::new("GET", "/openapi.json"));
        }
        let has_routes = !controller_registry.routes().is_empty();
        let controller_registry = Arc::new(controller_registry);
        let entity_registry = Arc::new(entity_registry);
        services.register_singleton::<Arc<EntityRegistry>, _>(move |_| entity_registry.clone());
        let services = services.build();
        let job_queue = match job_queue {
            JobQueueConfig::None => None,
            JobQueueConfig::Provided(queue) => Some(queue),
            JobQueueConfig::InMemory => services
                .resolve::<Arc<dyn JobQueue>>()
                .map(|queue| (*queue).clone()),
        };
        let address = address.unwrap_or_else(|| "0.0.0.0:8080".to_string());
        let pipeline = if has_routes {
            let mut middleware: Vec<Box<dyn DynMiddleware>> = Vec::new();
            middleware.push(Box::new(RoutingMiddleware::with_registry(
                router,
                Arc::clone(&controller_registry),
            )));
            middleware.extend(pipeline.into_middleware());
            middleware.push(Box::new(EndpointExecutionMiddleware::new()));
            Pipeline::from_middleware(middleware)
        } else {
            pipeline
        };

        let config = config_builder.build();
        Application::new(
            pipeline,
            services,
            hosted_services,
            job_queue,
            address,
            config,
        )
    }

    pub(crate) fn entity_registry_clone(&self) -> EntityRegistry {
        EntityRegistry::from_registry(&self.entity_registry)
    }
}
