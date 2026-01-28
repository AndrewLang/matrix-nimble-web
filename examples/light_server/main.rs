use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

use nimble_web::app::builder::AppBuilder;
use nimble_web::background::job::{BackgroundJob, JobContext, JobResult};
use nimble_web::background::job_queue::JobQueue;
use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::entity::entity::Entity;
use nimble_web::entity::operation::EntityOperation;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::result::Json;
use nimble_web::security::policy::Policy;
use nimble_web::validation::{ContextValidator, ValidationError};

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log::LevelFilter::Debug);
    let _ = builder.try_init();
}

#[derive(serde::Serialize)]
struct Health {
    status: &'static str,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct CreatePhoto {
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Photo {
    id: i64,
    name: String,
}

impl Entity for Photo {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "photo"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
}

impl Entity for User {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "User"
    }
}

use nimble_web::endpoint::route::EndpointRoute;

struct ApiController;

impl Controller for ApiController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/health", HealthHandler).build(),
            EndpointRoute::get("/photos", ListPhotosHandler).build(),
            EndpointRoute::post("/photos", CreatePhotoHandler)
                .validate(ContextValidator::new(|ctx| {
                    let payload: CreatePhoto = ctx.read_json()?;
                    if payload.name.trim().is_empty() {
                        return Err(ValidationError::new("name is required"));
                    }
                    Ok(())
                }))
                .build(),
            EndpointRoute::get("/photos/{id}", GetPhotoHandler).build(),
            EndpointRoute::get("/secure", SecureHandler)
                .with_policy(Policy::Authenticated)
                .build(),
        ]
    }
}

struct HealthHandler;

use async_trait::async_trait;

#[async_trait]
impl HttpHandler for HealthHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(Json(Health { status: "ok" })))
    }
}

struct ListPhotosHandler;

#[async_trait]
impl HttpHandler for ListPhotosHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(Json(vec![Photo {
            id: 1,
            name: "sunset".to_string(),
        }])))
    }
}

struct GetPhotoHandler;

#[async_trait]
impl HttpHandler for GetPhotoHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0);
        Ok(ResponseValue::new(Json(Photo {
            id,
            name: "example".to_string(),
        })))
    }
}

struct CreatePhotoHandler;

#[async_trait]
impl HttpHandler for CreatePhotoHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: CreatePhoto = context
            .read_json()
            .map_err(|err| PipelineError::message(err.message()))?;

        if let Some(queue) = context.services().resolve::<Arc<dyn JobQueue>>() {
            let job = CleanupJob {
                label: payload.name.clone(),
            };
            queue.enqueue(Box::new(job));
        }

        Ok(ResponseValue::new(Json(Photo {
            id: 1,
            name: payload.name,
        })))
    }
}

struct SecureHandler;

#[async_trait]
impl HttpHandler for SecureHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let subject = context
            .get::<IdentityContext>()
            .map(|identity| identity.identity().subject().to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        Ok(ResponseValue::new(format!("hello {}", subject)))
    }
}

struct CleanupJob {
    label: String,
}

impl BackgroundJob for CleanupJob {
    fn execute(&self, _ctx: JobContext) -> JobResult {
        println!("background cleanup for {}", self.label);
        JobResult::Success
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logging();

    log::info!("Starting light server example...");
    let mut builder = AppBuilder::new();
    builder
        .use_config("web.config.json")
        .use_env()
        .use_address_env("NIMBLE_ADDRESS")
        .use_authentication()
        .use_authorization()
        .use_validation()
        .use_in_memory_job_queue()
        .use_controller::<ApiController>()
        .use_memory_repository_with_data::<Photo>(vec![
            Photo {
                id: 1,
                name: "Sunset".to_string(),
            },
            Photo {
                id: 2,
                name: "Mountain".to_string(),
            },
        ])
        .use_memory_repository_with_data::<User>(vec![User {
            id: "1".to_string(),
            email: "admin@example.com".to_string(),
            password_hash: "hashed".to_string(),
        }])
        .use_entity::<Photo>()
        .use_entity_with_operations::<User>(&[EntityOperation::Get, EntityOperation::List])
        .route_get("/api/health", HealthHandler);

    let app = builder.build();

    let config = app.config();
    log::info!("Application configuration: {:?}", config);

    app.log_routes();

    app.start().await?;

    Ok(())
}
