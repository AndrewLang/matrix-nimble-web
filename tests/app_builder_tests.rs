use std::env;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use nimble_web::app::builder::AppBuilder;
use nimble_web::background::in_memory_queue::InMemoryJobQueue;
use nimble_web::background::job::{BackgroundJob, JobContext, JobResult};
use nimble_web::background::job_queue::JobQueue;
use nimble_web::controller::controller::Controller;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;

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
    fn register(registry: &mut ControllerRegistry) {
        registry.add("GET", "/hello", HelloHandler);
    }
}

struct HelloHandler;

impl HttpHandler for HelloHandler {
    async fn invoke(&self, _context: &mut nimble_web::http::context::HttpContext) -> Result<ResponseValue, PipelineError> {
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
    builder.add_controller::<HelloController>();
    let app = builder.build();

    let response = app.handle_http_request(HttpRequest::new("GET", "/hello"));
    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), &ResponseBody::Text("hello".to_string()));
}

#[test]
fn app_builder_without_routes_returns_not_found() {
    let app = AppBuilder::new().build();
    let response = app.handle_http_request(HttpRequest::new("GET", "/missing"));

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
    builder.add_job_queue(TestQueue::new(calls.clone()));
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

fn unique_suffix() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos()
}
