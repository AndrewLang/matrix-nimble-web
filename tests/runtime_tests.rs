use std::time::Duration;

use nimble_web::app::builder::AppBuilder;
use nimble_web::controller::controller::Controller;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;

struct HealthController;

impl Controller for HealthController {
    fn register(registry: &mut ControllerRegistry) {
        registry.add("GET", "/health", HealthHandler);
    }
}

struct HealthHandler;

impl HttpHandler for HealthHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

#[tokio::test]
async fn runtime_handles_request_and_shutdown() {
    let mut builder = AppBuilder::new();
    builder.use_address("127.0.0.1:0");
    builder.add_controller::<HealthController>();

    let app = builder.build();
    std::env::remove_var("NIMBLE_BOUND_ADDRESS");

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(app.start_with_shutdown(Box::pin(async move {
        let _ = rx.await;
    })));

    let addr = wait_for_bound_address().await;
    let url = format!("http://{}/health", addr);

    let response = reqwest::get(url).await.expect("request");
    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.expect("body");
    assert!(body.contains("ok"));

    let _ = tx.send(());
    let result = handle.await.expect("join");
    assert!(result.is_ok());
}

async fn wait_for_bound_address() -> String {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(value) = std::env::var("NIMBLE_BOUND_ADDRESS") {
            return value;
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for bound address");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
