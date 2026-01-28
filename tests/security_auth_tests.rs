use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_endpoint::HttpEndpoint;
use nimble_web::endpoint::http_endpoint_handler::HttpEndpointHandler;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::identity::context::IdentityContext;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::auth::AuthenticationMiddleware;
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

#[derive(Clone)]
struct TestEndpoint {
    trace: Trace,
}

#[async_trait]
impl HttpHandler for TestEndpoint {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        self.trace.push("endpoint");
        let subject = context
            .get::<IdentityContext>()
            .map(|id| id.identity().subject().to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        Ok(ResponseValue::new(subject))
    }
}

fn make_context(method: &str, path: &str) -> HttpContext {
    let request = HttpRequest::new(method, path);
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    HttpContext::new(request, services, config)
}

#[test]
fn authenticated_request_populates_identity() {
    let trace = Trace::new();
    let mut context = make_context("GET", "/secure");
    context
        .request_mut()
        .headers_mut()
        .insert("authorization", "Bearer test-user");

    let metadata = EndpointMetadata::new("GET", "/secure");
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        }),
        metadata,
    ));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    let identity = context
        .get::<IdentityContext>()
        .expect("identity context inserted");
    assert_eq!(identity.identity().subject(), "test-user");
    assert_eq!(trace.snapshot(), vec!["endpoint"]);
}

#[test]
fn missing_auth_header_allows_pipeline_continue() {
    let trace = Trace::new();
    let mut context = make_context("GET", "/secure");
    let metadata = EndpointMetadata::new("GET", "/secure");
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        }),
        metadata,
    ));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    let identity = context
        .get::<IdentityContext>()
        .expect("identity context inserted");
    assert!(!identity.is_authenticated());
}

#[test]
fn authorization_allows_access_with_policy() {
    let trace = Trace::new();
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        }),
        EndpointMetadata::new("GET", "/secure").require_policy(Policy::Authenticated),
    ));

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
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        }),
        EndpointMetadata::new("GET", "/secure").require_policy(Policy::Authenticated),
    ));

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
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(TestEndpoint {
            trace: trace.clone(),
        }),
        EndpointMetadata::new("GET", "/open"),
    ));

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
