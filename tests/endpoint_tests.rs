use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_endpoint::HttpEndpoint;
use nimble_web::endpoint::http_endpoint_handler::HttpEndpointHandler;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::endpoint::ws_endpoint::WsEndpoint;
use nimble_web::endpoint::ws_endpoint_handler::WsEndpointHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response::HttpResponse;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::middleware::routing::RoutingMiddleware;
use nimble_web::pipeline::middleware::Middleware;
use nimble_web::pipeline::next::Next;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
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

struct RecordingEndpoint {
    trace: Trace,
    status: u16,
}

#[async_trait]
impl HttpHandler for RecordingEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        self.trace.push("handled");
        let mut response = HttpResponse::new();
        response.set_status(self.status);
        Ok(ResponseValue::new(response))
    }
}

struct ParamEndpoint {
    trace: Trace,
}

#[async_trait]
impl HttpHandler for ParamEndpoint {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .map(String::as_str)
            .unwrap_or("missing");
        if id == "123" {
            self.trace.push("id:123");
        }
        Ok(ResponseValue::new(HttpResponse::new()))
    }
}

struct ErrorEndpoint;

#[async_trait]
impl HttpHandler for ErrorEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Err(PipelineError::message("boom"))
    }
}

struct MarkerMiddleware {
    trace: Trace,
}

#[async_trait]
impl Middleware for MarkerMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        self.trace.push("after");
        context.response_mut().set_status(202);
        next.run(context).await
    }
}

fn make_context(method: &str, path: &str) -> HttpContext {
    let request = HttpRequest::new(method, path);
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    HttpContext::new(request, services, config)
}

#[test]
fn http_endpoint_execution_invokes_handler() {
    let trace = Trace::new();
    let endpoint = RecordingEndpoint {
        trace: trace.clone(),
        status: 200,
    };

    let mut context = make_context("GET", "/photos");
    let metadata = EndpointMetadata::new("GET", "/photos");
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(endpoint),
        metadata,
    ));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["handled"]);
    assert_eq!(context.response().status(), 200);
}

#[test]
fn endpoint_not_present_allows_pipeline_to_continue() {
    let trace = Trace::new();

    let mut context = make_context("GET", "/photos");

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());
    pipeline.add(MarkerMiddleware {
        trace: trace.clone(),
    });

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["after"]);
    assert_eq!(context.response().status(), 202);
}

#[test]
fn http_endpoint_execution_continues_pipeline() {
    let trace = Trace::new();
    let endpoint = RecordingEndpoint {
        trace: trace.clone(),
        status: 201,
    };

    let mut context = make_context("GET", "/photos");
    let metadata = EndpointMetadata::new("GET", "/photos");
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(endpoint),
        metadata,
    ));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());
    pipeline.add(MarkerMiddleware {
        trace: trace.clone(),
    });

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["handled", "after"]);
    assert_eq!(context.response().status(), 202);
}

#[test]
fn websocket_endpoint_is_ignored() {
    let trace = Trace::new();

    let mut context = make_context("GET", "/ws");
    let metadata = EndpointMetadata::new("GET", "/ws");
    let endpoint = Arc::new(WsEndpoint::new(WsEndpointHandler, metadata));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());
    pipeline.add(MarkerMiddleware {
        trace: trace.clone(),
    });

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["after"]);
    assert_eq!(context.response().status(), 202);
}

#[test]
fn endpoint_executes_after_routing_and_sees_params() {
    let trace = Trace::new();
    let endpoint = ParamEndpoint {
        trace: trace.clone(),
    };

    let mut router = DefaultRouter::new();
    router.add_route(Route::new("GET", "/photos/{id}"));

    let mut context = make_context("GET", "/photos/123");
    let metadata = EndpointMetadata::new("GET", "/photos/{id}");
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(endpoint),
        metadata,
    ));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(RoutingMiddleware::new(router));
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(trace.snapshot(), vec!["id:123"]);
}

#[test]
fn endpoint_error_propagates_and_stops_pipeline() {
    let trace = Trace::new();

    let mut context = make_context("GET", "/photos");
    let metadata = EndpointMetadata::new("GET", "/photos");
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(ErrorEndpoint),
        metadata,
    ));
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());
    pipeline.add(MarkerMiddleware {
        trace: trace.clone(),
    });

    let result = pipeline.run(&mut context);

    assert!(matches!(result, Err(PipelineError::Message(msg)) if msg == "boom"));
    assert_eq!(trace.snapshot(), Vec::<&'static str>::new());
}
