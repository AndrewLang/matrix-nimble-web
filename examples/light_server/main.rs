use nimble_web::app::builder::AppBuilder;
use nimble_web::controller::controller::Controller;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::result::Json;

#[derive(serde::Serialize)]
struct Health {
    status: &'static str,
}

struct HealthController;

impl Controller for HealthController {
    fn register(registry: &mut ControllerRegistry) {
        registry.add("GET", "/health", HealthHandler);
    }
}

struct HealthHandler;

impl HttpHandler for HealthHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(Json(Health { status: "ok" })))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = AppBuilder::new();
    builder.use_address_env("NIMBLE_ADDRESS");
    builder.add_controller::<HealthController>();
    let app = builder.build();
    app.start().await?;

    Ok(())
}
