use std::error::Error;
use std::sync::Arc;

use nimble_web::app::builder::AppBuilder;
use nimble_web::background::job::{BackgroundJob, JobContext, JobResult};
use nimble_web::background::job_queue::JobQueue;
use nimble_web::controller::controller::Controller;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::entity::entity::Entity;
use nimble_web::http::context::HttpContext;
use nimble_web::openapi::OpenApiSchema;
use nimble_web::openapi::Schema;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::result::Json;
use nimble_web::security::auth::User;
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

#[derive(serde::Serialize, Clone)]
struct Photo {
    id: i64,
    name: String,
}

struct PhotoEntity;

impl Entity for PhotoEntity {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        static ID: i64 = 0;
        &ID
    }

    fn name() -> &'static str {
        "photo"
    }

    fn plural_name() -> &'static str {
        "photos"
    }
}

impl OpenApiSchema for CreatePhoto {
    fn schema() -> Schema {
        let mut properties = std::collections::HashMap::new();
        properties.insert("name".to_string(), Schema::string());
        Schema::object(properties, vec!["name".to_string()])
    }

    fn schema_name() -> String {
        "CreatePhoto".to_string()
    }
}

impl OpenApiSchema for Photo {
    fn schema() -> Schema {
        let mut properties = std::collections::HashMap::new();
        properties.insert("id".to_string(), Schema::integer_with_format("int64"));
        properties.insert("name".to_string(), Schema::string());
        Schema::object(properties, vec!["id".to_string(), "name".to_string()])
    }

    fn schema_name() -> String {
        "Photo".to_string()
    }
}

struct ApiController;

impl Controller for ApiController {
    fn register(registry: &mut ControllerRegistry) {
        registry.add("GET", "/health", HealthHandler);

        registry
            .get("/photos", ListPhotosHandler)
            .query::<i32>("page")
            .query::<i32>("pageSize")
            .summary("List photos")
            .tag("photos")
            .register();

        registry
            .post("/photos", CreatePhotoHandler)
            .body::<CreatePhoto>()
            .responds::<Photo>(200)
            .summary("Create a photo")
            .tag("photos")
            .validate(ContextValidator::new(|ctx| {
                let payload: CreatePhoto = ctx.read_json()?;
                if payload.name.trim().is_empty() {
                    return Err(ValidationError::new("name is required"));
                }
                Ok(())
            }))
            .register();

        registry
            .get("/photos/{id}", GetPhotoHandler)
            .param::<i64>("id")
            .responds::<Photo>(200)
            .summary("Get photo by id")
            .tag("photos")
            .register();

        registry.add_with_policy("GET", "/secure", SecureHandler, Policy::Authenticated);
    }
}

struct HealthHandler;

impl HttpHandler for HealthHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(Json(Health { status: "ok" })))
    }
}

struct ListPhotosHandler;

impl HttpHandler for ListPhotosHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(Json(vec![Photo {
            id: 1,
            name: "sunset".to_string(),
        }])))
    }
}

struct GetPhotoHandler;

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

impl HttpHandler for SecureHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let user = context
            .get::<User>()
            .map(|user| user.id.clone())
            .unwrap_or_else(|| "anonymous".to_string());
        Ok(ResponseValue::new(format!("hello {}", user)))
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

    let mut builder = AppBuilder::new();
    builder
        .use_config("web.config.json")
        .use_env()
        .use_address_env("NIMBLE_ADDRESS")
        .use_authentication()
        .use_authorization()
        .use_validation()
        .use_in_memory_job_queue()
        .add_controller::<ApiController>()
        .add_entity::<PhotoEntity>();

    let app = builder.build();

    let config = app.config();
    log::info!("Application configuration: {:?}", config);

    app.start().await?;

    Ok(())
}
