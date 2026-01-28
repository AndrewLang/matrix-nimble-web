use std::sync::{Arc, Mutex};

use nimble_web::app::builder::AppBuilder;
use nimble_web::background::job::{BackgroundJob, JobContext, JobResult};
use nimble_web::controller::controller::Controller;

use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::result::IntoResponse;
use nimble_web::result::Json;
use nimble_web::testkit::app::TestApp;
use nimble_web::testkit::background::BackgroundTestkit;
use nimble_web::testkit::entity::{EntityRegistryAssertions, EntityTestkit};
use nimble_web::testkit::response::ResponseAssertions;
use nimble_web::testkit::services::TestServices;

#[derive(Debug, Clone)]
struct TestEntity {
    id: i64,
}

impl nimble_web::entity::entity::Entity for TestEntity {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "test_entity"
    }
}

struct HelloController;

use nimble_web::controller::registry::EndpointRoute;

impl Controller for HelloController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::get("/hello", HelloHandler).build()]
    }
}

struct HelloHandler;

impl HttpHandler for HelloHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

struct CountJob {
    calls: Arc<Mutex<u32>>,
}

impl CountJob {
    fn new(calls: Arc<Mutex<u32>>) -> Self {
        Self { calls }
    }
}

impl BackgroundJob for CountJob {
    fn execute(&self, _ctx: JobContext) -> JobResult {
        let mut guard = self.calls.lock().expect("calls lock");
        *guard += 1;
        JobResult::Success
    }
}

#[tokio::test]
async fn testapp_run_async_matches_run() {
    let response = TestApp::new()
        .add_controller::<HelloController>()
        .run_async(HttpRequest::new("GET", "/hello"))
        .await;

    response.assert_status(200);
    response.assert_body("ok");
}

#[test]
fn background_testkit_enqueues_and_runs_jobs() {
    let calls = Arc::new(Mutex::new(0));
    let mut app = TestApp::new();
    app.enqueue(CountJob::new(calls.clone()));
    let results = app.run_background_jobs();

    assert_eq!(results.len(), 1);
    assert_eq!(*calls.lock().expect("calls lock"), 1);
}

#[test]
fn entity_testkit_registers_entities() {
    let mut builder = AppBuilder::new();
    builder.use_entity::<TestEntity>();

    builder.assert_has_entity("test_entity");
    let registry = builder.entity_registry();
    registry.assert_entity("test_entity");
}

#[test]
fn test_services_registers_singletons() {
    let services = TestServices::new()
        .add_singleton(42u32)
        .override_singleton(7u32)
        .build();

    let value = services.resolve::<u32>().expect("u32");
    assert_eq!(*value, 7);
}

#[test]
fn response_assertions_json_works() {
    let mut context = HttpContext::new(
        HttpRequest::new("GET", "/json"),
        ServiceContainer::new().build(),
        nimble_web::config::ConfigBuilder::new().build(),
    );
    Json(serde_json::json!({"ok": true})).into_response(&mut context);
    let response = context.response().clone();

    let parsed: serde_json::Value = response.assert_json();
    assert_eq!(parsed["ok"], true);
}
