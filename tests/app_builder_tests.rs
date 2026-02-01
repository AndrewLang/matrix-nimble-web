use std::env;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use nimble_web::app::application::Application;
use nimble_web::app::builder::AppBuilder;
use nimble_web::background::in_memory_queue::InMemoryJobQueue;
use nimble_web::background::job::{BackgroundJob, JobContext, JobResult};
use nimble_web::background::job_queue::JobQueue;
use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::entity::operation::EntityOperation;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response::HttpResponse;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::routing::route::Route;
use nimble_web::security::policy::Policy;
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock")
}

struct EnvGuard {
    key: String,
    prev: Option<String>,
}

impl EnvGuard {
    fn set(key: &str, value: &str) -> Self {
        let prev = env::var(key).ok();
        env::set_var(key, value);
        Self {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.prev {
            env::set_var(&self.key, value);
        } else {
            env::remove_var(&self.key);
        }
    }
}

struct HelloController;

impl Controller for HelloController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::get("/hello", HelloHandler).build()]
    }
}

use async_trait::async_trait;

// ...

struct HelloHandler;

#[async_trait]
impl HttpHandler for HelloHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("hello"))
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

#[derive(Serialize, Deserialize)]
struct AlbumEntity;

impl Entity for AlbumEntity {
    type Id = String;

    fn id(&self) -> &Self::Id {
        unimplemented!()
    }

    fn name() -> &'static str {
        "Album"
    }
}

struct TestQueue {
    calls: Arc<Mutex<u32>>,
}

impl TestQueue {
    fn new(calls: Arc<Mutex<u32>>) -> Self {
        Self { calls }
    }
}

impl JobQueue for TestQueue {
    fn enqueue(&self, _job: Box<dyn BackgroundJob>) {
        let mut guard = self.calls.lock().expect("calls lock");
        *guard += 1;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn app_builder_routes_requests_with_controller() {
    let mut builder = AppBuilder::new();
    builder.use_controller::<HelloController>();
    let app = builder.build();

    let response = handle_request(&app, HttpRequest::new("GET", "/hello"));
    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), &ResponseBody::Text("hello".to_string()));
}

#[test]
fn app_builder_without_routes_returns_not_found() {
    let app = AppBuilder::new().build();
    let response = handle_request(&app, HttpRequest::new("GET", "/missing"));

    assert_eq!(response.status(), 404);
    assert_eq!(response.body(), &ResponseBody::Empty);
}

#[test]
fn app_builder_loads_relative_config_from_exe_dir() {
    let exe = env::current_exe().expect("current exe");
    let base = exe.parent().expect("exe dir");
    let filename = format!("nimble-config-{}.json", unique_suffix());
    let path = base.join(&filename);
    let content = "{\"server\":{\"port\":5050}}";

    std::fs::write(&path, content).expect("write config");

    let mut builder = AppBuilder::new();
    builder.use_config(&filename);
    let app = builder.build();

    assert_eq!(app.config().get("server.port"), Some("5050"));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn app_builder_loads_env_config_with_prefix() {
    let _lock = env_lock();
    let _guard = EnvGuard::set("NIMBLE_FEATURE_ENABLED", "true");

    let mut builder = AppBuilder::new();
    builder.use_env_prefix("NIMBLE_");
    let app = builder.build();

    assert_eq!(app.config().get_bool("feature.enabled"), Some(true));
}

#[test]
fn app_builder_registers_in_memory_job_queue() {
    let mut builder = AppBuilder::new();
    builder.use_in_memory_job_queue();
    let app = builder.build();

    let queue = app
        .services()
        .resolve::<Arc<dyn JobQueue>>()
        .expect("queue");
    let in_memory = queue
        .as_any()
        .downcast_ref::<InMemoryJobQueue>()
        .expect("in memory queue");

    let calls = Arc::new(Mutex::new(0));
    in_memory.enqueue(Box::new(CountJob::new(calls.clone())));
    let results = in_memory.run_all();

    assert_eq!(results.len(), 1);
    assert_eq!(*calls.lock().expect("calls lock"), 1);
}

#[test]
fn app_builder_registers_provided_job_queue() {
    let calls = Arc::new(Mutex::new(0));
    let mut builder = AppBuilder::new();
    builder.use_job_queue(TestQueue::new(calls.clone()));
    let app = builder.build();

    let queue = app
        .services()
        .resolve::<Arc<dyn JobQueue>>()
        .expect("queue");
    let test_queue = queue
        .as_any()
        .downcast_ref::<TestQueue>()
        .expect("test queue");
    test_queue.enqueue(Box::new(CountJob::new(calls.clone())));

    assert_eq!(*calls.lock().expect("calls lock"), 1);
}

#[test]
fn app_exposes_router_with_registered_routes() {
    let mut builder = AppBuilder::new();
    builder.use_controller::<HelloController>();
    let app = builder.build();

    let router = app.router();
    let routes = router.routes();

    assert!(routes
        .iter()
        .any(|r| r.method() == "GET" && r.path() == "/hello"));

    app.log_routes();
}

#[test]
fn entity_operations_with_policy_attach_policy() {
    let mut builder = AppBuilder::new();
    builder.use_entity_with_operations_and_policy::<AlbumEntity>(
        &[EntityOperation::List, EntityOperation::Get],
        Policy::Authenticated,
    );

    let registry = builder.endpoint_registry_clone();
    let list_route = Route::new("GET", "/api/albums/{page}/{pageSize}");
    let get_route = Route::new("GET", "/api/albums/{id}");

    let list_endpoint = registry
        .find_endpoint(&list_route)
        .expect("list endpoint should be registered");
    assert_eq!(
        list_endpoint
            .metadata()
            .policy()
            .cloned()
            .expect("list policy"),
        Policy::Authenticated
    );

    let get_endpoint = registry
        .find_endpoint(&get_route)
        .expect("get endpoint should be registered");
    assert_eq!(
        get_endpoint
            .metadata()
            .policy()
            .cloned()
            .expect("get policy"),
        Policy::Authenticated
    );
}

fn unique_suffix() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos()
}

fn handle_request(app: &Application, request: HttpRequest) -> HttpResponse {
    let runtime = Runtime::new().expect("runtime");
    runtime.block_on(app.handle_http_request(request))
}
