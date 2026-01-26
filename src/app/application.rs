use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::background::hosted_service::{HostedServiceContext, HostedServiceHost};
use crate::background::in_memory_queue::InMemoryJobQueue;
use crate::background::job_queue::JobQueue;
use crate::background::runner::JobQueueRunner;
use crate::config::Configuration;
use crate::di::ServiceProvider;
use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::pipeline::pipeline::Pipeline;
use crate::runtime::hyper_runtime::HyperRuntime;
use crate::runtime::runtime::Runtime;

pub struct Application {
    pipeline: Pipeline,
    services: ServiceProvider,
    hosted_services: HostedServiceHost,
    job_queue: Option<Arc<dyn JobQueue>>,
    address: String,
    config: Configuration,
}

impl Application {
    pub(crate) fn new(
        pipeline: Pipeline,
        services: ServiceProvider,
        hosted_services: HostedServiceHost,
        job_queue: Option<Arc<dyn JobQueue>>,
        address: String,
        config: Configuration,
    ) -> Self {
        Self {
            pipeline,
            services,
            hosted_services,
            job_queue,
            address,
            config,
        }
    }

    pub async fn start(self) -> Result<(), AppError> {
        let addr = self.parse_address()?;
        log::info!("Start application at {}", addr);
        let wants_random = addr.port() == 0;

        let context = match &self.job_queue {
            Some(queue) => {
                HostedServiceContext::with_job_queue(self.services.clone(), queue.clone())
            }
            None => HostedServiceContext::new(self.services.clone()),
        };
        self.hosted_services.start(context);

        let runtime = HyperRuntime::new();
        let shutdown = Self::shutdown_signal();
        let app = Arc::new(self);

        log::debug!("Starting runtime...");
        runtime
            .run(addr, Arc::clone(&app), Box::pin(shutdown), wants_random)
            .await?;
        log::info!("Shutting down application");
        app.shutdown();
        app.flush_jobs();
        Ok(())
    }

    pub fn shutdown(&self) {
        self.hosted_services.stop();
    }

    pub fn services(&self) -> &ServiceProvider {
        &self.services
    }

    pub(crate) fn create_context(&self, request: HttpRequest) -> HttpContext {
        let services = self.services.clone();
        HttpContext::new(request, services, self.config.clone())
    }

    pub(crate) fn handle_context(&self, context: &mut HttpContext) {
        let _ = self.pipeline.run(context);
    }

    pub fn handle_http(&self, request: HttpRequest) -> HttpResponse {
        log::debug!(
            "Handling HTTP request: {} {}",
            request.method(),
            request.path()
        );

        let mut context = self.create_context(request);
        self.handle_context(&mut context);
        context.response().clone()
    }

    fn parse_address(&self) -> Result<SocketAddr, AppError> {
        self.address
            .parse()
            .map_err(|_| AppError::InvalidAddress(self.address.clone()))
    }

    fn flush_jobs(&self) {
        let Some(queue) = self.job_queue.as_ref() else {
            return;
        };

        if let Some(in_memory) = queue.as_any().downcast_ref::<InMemoryJobQueue>() {
            in_memory.run_all();
            return;
        }

        if let Some(runner) = queue.as_any().downcast_ref::<JobQueueRunner>() {
            runner.run_pending_jobs();
        }
    }

    async fn shutdown_signal() {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut terminate = signal(SignalKind::terminate()).expect("signal handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = terminate.recv() => {},
            }
        }

        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    InvalidAddress(String),
    Runtime(String),
}

impl AppError {
    pub(crate) fn runtime(stage: &str, err: impl Display) -> Self {
        AppError::Runtime(format!("{}: {}", stage, err))
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AppError::InvalidAddress(address) => {
                write!(f, "invalid address: {}", address)
            }
            AppError::Runtime(message) => write!(f, "runtime error: {}", message),
        }
    }
}

impl Error for AppError {}
