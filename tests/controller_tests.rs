use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nimble_web::config::ConfigBuilder;
use nimble_web::controller::controller::Controller;
use nimble_web::controller::invoker::ControllerInvokerMiddleware;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::registry::EndpointRegistry;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::middleware::routing::RoutingMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
use nimble_web::result::HttpError;
use nimble_web::routing::default_router::DefaultRouter;
use nimble_web::routing::router::Router;

#[derive(Clone)]
struct Trace {
    steps: Arc<Mutex<Vec<&'static str>>>,
}

impl Trace {
    fn new() -> Self {
        Self {
            steps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn push(&self, label: &'static str) {
        self.steps.lock().expect("trace lock").push(label);
    }

    fn snapshot(&self) -> Vec<&'static str> {
        self.steps.lock().expect("trace lock").clone()
    }
}

thread_local! {
    static TEST_TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}

fn set_test_trace(trace: Trace) {
    TEST_TRACE.with(|cell| {
        *cell.borrow_mut() = Some(trace);
    });
}

fn test_trace() -> Trace {
    TEST_TRACE.with(|cell| cell.borrow().clone().expect("test trace set"))
}

use nimble_web::endpoint::route::EndpointRoute;

struct TestController;

impl Controller for TestController {
    fn routes() -> Vec<EndpointRoute> {
        let trace = test_trace();
        vec![EndpointRoute::get("/photos", TestEndpoint { trace }).build()]
    }
}

struct ParamController;

impl Controller for ParamController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::get("/photos/{id}", ParamEndpoint).build()]
    }
}

struct ErrorController;

impl Controller for ErrorController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::get("/error", ErrorEndpoint).build()]
    }
}

#[derive(Clone)]
struct TestEndpoint {
    trace: Trace,
}

#[async_trait(?Send)]
impl HttpHandler for TestEndpoint {
    async fn invoke(&self, _context: &HttpContext) -> Result<ResponseValue, PipelineError> {
        self.trace.push("invoked");
        Ok(ResponseValue::new("ok"))
    }
}

struct ParamEndpoint;

#[async_trait(?Send)]
impl HttpHandler for ParamEndpoint {
    async fn invoke(&self, context: &HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .map(String::as_str)
            .unwrap_or("missing");
        Ok(ResponseValue::new(id.to_string()))
    }
}

struct ErrorEndpoint;

#[async_trait(?Send)]
impl HttpHandler for ErrorEndpoint {
    async fn invoke(&self, _context: &HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(HttpError::new(404, "not found")))
    }
}

fn make_context(method: &str, path: &str) -> HttpContext {
    let request = HttpRequest::new(method, path);
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    HttpContext::new(request, services, config)
}

#[test]
fn controller_registration_adds_routes_and_endpoints() {
    let trace = Trace::new();
    set_test_trace(trace.clone());
    let mut registry = EndpointRegistry::new();

    registry.register::<TestController>();

    assert_eq!(registry.routes().len(), 1);
    assert_eq!(registry.endpoints().len(), 1);
}

#[test]
fn controller_action_invocation_populates_response() {
    let trace = Trace::new();
    set_test_trace(trace.clone());
    let mut registry = EndpointRegistry::new();
    registry.register::<TestController>();

    let mut router = DefaultRouter::new();
    for route in registry.routes() {
        router.add_route(route.clone());
    }

    let mut context = make_context("GET", "/photos");

    let mut pipeline = Pipeline::new();
    pipeline.add(RoutingMiddleware::new(router));
    pipeline.add(ControllerInvokerMiddleware::new(Arc::new(registry)));
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["invoked"]);
    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("ok".to_string())
    );
}

#[test]
fn route_params_passed_to_controller() {
    let mut registry = EndpointRegistry::new();
    registry.register::<ParamController>();

    let mut router = DefaultRouter::new();
    for route in registry.routes() {
        router.add_route(route.clone());
    }

    let mut context = make_context("GET", "/photos/123");

    let mut pipeline = Pipeline::new();
    pipeline.add(RoutingMiddleware::new(router));
    pipeline.add(ControllerInvokerMiddleware::new(Arc::new(registry)));
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("123".to_string())
    );
}

#[test]
fn multiple_controllers_register_routes_without_conflicts() {
    let trace = Trace::new();
    set_test_trace(trace.clone());

    let mut registry = EndpointRegistry::new();
    registry.register::<TestController>();
    registry.register::<ParamController>();

    assert_eq!(registry.routes().len(), 2);
}

#[test]
fn controller_error_propagates() {
    let mut registry = EndpointRegistry::new();
    registry.register::<ErrorController>();

    let mut router = DefaultRouter::new();
    for route in registry.routes() {
        router.add_route(route.clone());
    }

    let mut context = make_context("GET", "/error");

    let mut pipeline = Pipeline::new();
    pipeline.add(RoutingMiddleware::new(router));
    pipeline.add(ControllerInvokerMiddleware::new(Arc::new(registry)));
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(context.response().status(), 404);
}
