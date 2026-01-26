use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use nimble_web::config::ConfigBuilder;
use nimble_web::controller::controller::Controller;
use nimble_web::controller::invoker::ControllerInvokerMiddleware;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::endpoint::Endpoint;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::middleware::routing::RoutingMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
use nimble_web::result::HttpError;
use nimble_web::routing::default_router::DefaultRouter;
use nimble_web::routing::route::Route;
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

struct TestController;

impl Controller for TestController {
    fn register(registry: &mut ControllerRegistry) {
        let trace = test_trace();
        let metadata = EndpointMetadata::new("GET", "/photos");
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(TestEndpoint { trace })),
            metadata,
        );
        registry.add_route(Route::new("GET", "/photos"), endpoint);
    }
}

struct ParamController;

impl Controller for ParamController {
    fn register(registry: &mut ControllerRegistry) {
        let metadata = EndpointMetadata::new("GET", "/photos/{id}");
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(ParamEndpoint)),
            metadata,
        );
        registry.add_route(Route::new("GET", "/photos/{id}"), endpoint);
    }
}

struct ErrorController;

impl Controller for ErrorController {
    fn register(registry: &mut ControllerRegistry) {
        let metadata = EndpointMetadata::new("GET", "/error");
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(ErrorEndpoint)),
            metadata,
        );
        registry.add_route(Route::new("GET", "/error"), endpoint);
    }
}

struct TestEndpoint {
    trace: Trace,
}

impl HttpHandler for TestEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        self.trace.push("invoked");
        Ok(ResponseValue::new("ok"))
    }
}

struct ParamEndpoint;

impl HttpHandler for ParamEndpoint {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .map(String::as_str)
            .unwrap_or("missing");
        Ok(ResponseValue::new(id.to_string()))
    }
}

struct ErrorEndpoint;

impl HttpHandler for ErrorEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let error = HttpError::new(404, "not found");
        Ok(ResponseValue::new(error))
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
    let mut registry = ControllerRegistry::new();

    registry.register::<TestController>();

    assert_eq!(registry.routes().len(), 1);
    assert_eq!(registry.endpoints().len(), 1);
}

#[test]
fn controller_action_invocation_populates_response() {
    let trace = Trace::new();
    set_test_trace(trace.clone());
    let mut registry = ControllerRegistry::new();
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
    let mut registry = ControllerRegistry::new();
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

    let mut registry = ControllerRegistry::new();
    registry.register::<TestController>();
    registry.register::<ParamController>();

    assert_eq!(registry.routes().len(), 2);
}

#[test]
fn controller_error_propagates() {
    let mut registry = ControllerRegistry::new();
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
