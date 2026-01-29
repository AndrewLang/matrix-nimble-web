use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::Hash;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use crate::app::application::Application;
use crate::background::config::JobQueueConfig;
use crate::background::hosted_service::{HostedService, HostedServiceHost};
use crate::background::in_memory_queue::InMemoryJobQueue;
use crate::background::job_queue::JobQueue;
use crate::config::ConfigBuilder;
use crate::controller::controller::Controller;
use crate::data::memory_repository::MemoryRepository;
use crate::data::provider::DataProvider;
use crate::di::ServiceContainer;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::registry::EndpointRegistry;
use crate::entity::entity::Entity;
use crate::entity::hooks::{DefaultEntityHooks, EntityHooks};
use crate::entity::operation::{EntityOperation, OperationHandler};
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
use crate::Configuration;

#[cfg(feature = "redis")]
use crate::redis::RedisModule;

#[cfg(feature = "postgres")]
use {crate::data::postgres::migration::PostgresMigrator, sqlx::postgres::PgPoolOptions};

pub struct AppBuilder {
    pipeline: Pipeline,
    endpoint_registry: EndpointRegistry,
    router: DefaultRouter,
    services: ServiceContainer,
    hosted_services: HostedServiceHost,
    job_queue: JobQueueConfig,
    entity_registry: EntityRegistry,
    address: Option<String>,
    config_builder: ConfigBuilder,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            endpoint_registry: EndpointRegistry::new(),
            router: DefaultRouter::new(),
            services: ServiceContainer::new(),
            hosted_services: HostedServiceHost::new(),
            job_queue: JobQueueConfig::None,
            entity_registry: EntityRegistry::new(),
            address: None,
            config_builder: ConfigBuilder::new(),
        }
    }

    pub fn routes(&mut self) -> &mut EndpointRegistry {
        &mut self.endpoint_registry
    }

    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.pipeline.add(middleware);
        self
    }

    pub fn use_controller<T: Controller>(&mut self) -> &mut Self {
        let routes = T::routes();
        for route in routes {
            self.endpoint_registry.add_endpoint_route(route);
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

    pub fn use_hosted_service<T: HostedService>(&mut self, service: T) -> &mut Self {
        self.hosted_services.add(service);
        self
    }

    pub fn use_job_queue<T>(&mut self, queue: T) -> &mut Self
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

        log::debug!("Using config file at {}", path.display());

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
            endpoint_registry,
            mut router,
            mut services,
            hosted_services,
            job_queue,
            entity_registry,
            address,
            config_builder,
        } = self;

        for route in endpoint_registry.routes() {
            router.add_route(route.clone());
        }

        let has_routes = !endpoint_registry.routes().is_empty();
        let endpoint_registry = Arc::new(endpoint_registry);
        let entity_registry = Arc::new(entity_registry);
        services.register_singleton::<Arc<EntityRegistry>, _>(move |_| entity_registry.clone());

        let config = config_builder.build();
        let config_clone = config.clone();
        services.register_singleton::<Configuration, _>(move |_| config_clone.clone());

        #[cfg(feature = "redis")]
        RedisModule::register(&mut services, &config);

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
            let mut middlewares: Vec<Box<dyn DynMiddleware>> = Vec::new();
            middlewares.push(Box::new(RoutingMiddleware::with_registry(
                router.clone(),
                Arc::clone(&endpoint_registry),
            )));
            middlewares.extend(pipeline.into_middleware());
            middlewares.push(Box::new(EndpointExecutionMiddleware::new()));

            Pipeline::from_middleware(middlewares)
        } else {
            pipeline
        };

        Application::new(
            pipeline,
            services,
            hosted_services,
            job_queue,
            address,
            config,
            router,
        )
    }

    pub(crate) fn entity_registry_clone(&self) -> EntityRegistry {
        EntityRegistry::from_registry(&self.entity_registry)
    }
}

impl AppBuilder {
    pub fn route_get<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.endpoint_registry.get(path, handler);
        self
    }

    pub fn route_post<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.endpoint_registry.post(path, handler);
        self
    }

    pub fn route_put<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.endpoint_registry.put(path, handler);
        self
    }

    pub fn route_delete<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.endpoint_registry.delete(path, handler);
        self
    }
}

#[cfg(feature = "postgres")]
impl AppBuilder {
    pub fn use_postgres(&mut self) -> &mut Self {
        log::info!("Registering pool options");
        self.services.register_singleton(|provider| {
            let config = provider
                .resolve::<Configuration>()
                .expect("configuration not registered");
            let pg_config = config.postgres_config();

            PgPoolOptions::new()
                .max_connections(pg_config.pool_size)
                .connect_lazy(&pg_config.url)
                .expect("❌  Invalid postgres configuration")
        });

        log::info!("Registering postgres config");
        self.services.register_singleton(|provider| {
            let config = provider
                .resolve::<Configuration>()
                .expect("❌  Postgres configuration not registered");
            config.postgres_config()
        });

        log::info!("Registering migrator");
        self.services.register_singleton(|provider| {
            let pool = provider
                .resolve::<sqlx::PgPool>()
                .expect("❌  Postgres pool not registered");
            PostgresMigrator::new(pool.as_ref().clone())
        });

        self
    }
}

impl AppBuilder {
    pub fn use_entity<E>(&mut self) -> &mut Self
    where
        E: Entity + Serialize + DeserializeOwned + 'static,
        E::Id: FromStr + Send + Sync + 'static,
    {
        self.entity_registry.register::<E>();
        self.use_entity_with_operations::<E>(EntityOperation::all())
    }

    pub fn use_entity_with_operations<E>(&mut self, operations: &[EntityOperation]) -> &mut Self
    where
        E: Entity + Serialize + DeserializeOwned + 'static,
        E::Id: FromStr + Send + Sync + 'static,
    {
        self.use_entity_with_hooks::<E, DefaultEntityHooks>(DefaultEntityHooks, operations)
    }

    pub fn use_entity_with_hooks<E, H>(
        &mut self,
        hooks: H,
        operations: &[EntityOperation],
    ) -> &mut Self
    where
        E: Entity + Serialize + DeserializeOwned + 'static,
        E::Id: FromStr + Send + Sync + 'static,
        H: EntityHooks<E> + 'static,
    {
        let hooks = Arc::new(hooks);
        let plural = E::plural_name().to_lowercase();
        let base_path = format!("api/{}", plural);

        for op in operations {
            match op {
                EntityOperation::List => {
                    self.route_get(
                        &format!("{}/{{page}}/{{pageSize}}", base_path),
                        OperationHandler::new(EntityOperation::List, hooks.clone()),
                    );
                }
                EntityOperation::Get => {
                    self.route_get(
                        &format!("{}/{{id}}", base_path),
                        OperationHandler::new(EntityOperation::Get, hooks.clone()),
                    );
                }
                EntityOperation::Create => {
                    self.route_post(
                        &base_path,
                        OperationHandler::new(EntityOperation::Create, hooks.clone()),
                    );
                }
                EntityOperation::Update => {
                    self.route_put(
                        &base_path,
                        OperationHandler::new(EntityOperation::Update, hooks.clone()),
                    );
                }
                EntityOperation::Delete => {
                    self.route_delete(
                        &format!("{}/{{id}}", base_path),
                        OperationHandler::new(EntityOperation::Delete, hooks.clone()),
                    );
                }
            }
        }

        self
    }

    pub fn use_memory_repository<E>(&mut self) -> &mut Self
    where
        E: Entity + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
        E::Id: FromStr + Hash + Eq + Clone + Send + Sync + 'static,
    {
        let repo = MemoryRepository::<E>::new();
        self.services
            .register_singleton::<Arc<dyn DataProvider<E>>, _>(move |_| Arc::new(repo.clone()));
        self
    }

    pub fn use_memory_repository_with_data<E>(&mut self, data: Vec<E>) -> &mut Self
    where
        E: Entity + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
        E::Id: FromStr + Hash + Eq + Clone + Send + Sync + 'static,
    {
        let repo = MemoryRepository::<E>::new();
        repo.seed(data);
        self.services
            .register_singleton::<Arc<dyn DataProvider<E>>, _>(move |_| Arc::new(repo.clone()));
        self
    }
}
