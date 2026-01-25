use std::sync::{Arc, Mutex};

use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::endpoint::Endpoint;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::auth::{AuthenticationMiddleware, User};
use nimble_web::security::policy::{AuthorizationMiddleware, Policy};

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

struct TestEndpoint {
    trace: Trace,
}

impl HttpHandler for TestEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        self.trace.push("endpoint");
        Ok(ResponseValue::new("ok"))
    }
}

fn make_context(method: &str, path: &str) -> HttpContext {
    let request = HttpRequest::new(method, path);
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    HttpContext::new(request, services, config)
}

#[test]
fn authentication_attaches_user() {
    let mut context = make_context("GET", "/secure");
    context
        .request_mut()
        .headers_mut()
        .insert("authorization", "Bearer test-user");

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert!(context.get::<User>().is_some());
}

#[test]
fn missing_auth_header_allows_pipeline_continue() {
    let mut context = make_context("GET", "/secure");

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert!(context.get::<User>().is_none());
}

#[test]
fn authorization_allows_access_with_policy() {
    let trace = Trace::new();
    let endpoint = Endpoint::new(
        EndpointKind::Http(HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        })),
        EndpointMetadata::new("GET", "/secure").require_policy(Policy::Authenticated),
    );

    let mut context = make_context("GET", "/secure");
    context
        .request_mut()
        .headers_mut()
        .insert("authorization", "Bearer test-user");
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());
    pipeline.add(AuthorizationMiddleware::new());
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["endpoint"]);
    assert_eq!(context.response().status(), 200);
}

#[test]
fn authorization_denies_access_without_user() {
    let trace = Trace::new();
    let endpoint = Endpoint::new(
        EndpointKind::Http(HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        })),
        EndpointMetadata::new("GET", "/secure").require_policy(Policy::Authenticated),
    );

    let mut context = make_context("GET", "/secure");
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());
    pipeline.add(AuthorizationMiddleware::new());
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), Vec::<&'static str>::new());
    assert_eq!(context.response().status(), 403);
}

#[test]
fn authorization_allows_when_no_policy() {
    let trace = Trace::new();
    let endpoint = Endpoint::new(
        EndpointKind::Http(HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        })),
        EndpointMetadata::new("GET", "/open"),
    );

    let mut context = make_context("GET", "/open");
    context.set_endpoint(endpoint.clone());

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthorizationMiddleware::new());
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["endpoint"]);
    assert_eq!(context.response().status(), 200);
}
