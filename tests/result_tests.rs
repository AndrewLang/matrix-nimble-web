use std::collections::HashMap;

use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::endpoint::Endpoint;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::{IntoResponse, ResponseValue};
use nimble_web::result::{HttpError, Json};

#[test]
fn text_response_from_str() {
    let mut context = make_context("GET", "/text");
    "hello".into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("hello".to_string())
    );
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("text/plain; charset=utf-8")
    );
}

#[test]
fn text_response_from_string() {
    let mut context = make_context("GET", "/text");
    "hello".to_string().into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("hello".to_string())
    );
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("text/plain; charset=utf-8")
    );
}

#[test]
fn json_response_from_wrapper() {
    let mut payload = HashMap::new();
    payload.insert("id", 123);

    let mut context = make_context("GET", "/json");
    Json(payload).into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("{\"id\":123}".to_string())
    );
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("application/json")
    );
}

#[test]
fn result_into_response_ok_and_err() {
    let mut ok_context = make_context("GET", "/result");
    Ok::<_, HttpError>("ok").into_response(&mut ok_context);
    assert_eq!(ok_context.response().status(), 200);
    assert_eq!(
        ok_context.response().body(),
        &ResponseBody::Text("ok".to_string())
    );

    let mut err_context = make_context("GET", "/result");
    Err::<&str, _>(HttpError::new(500, "error")).into_response(&mut err_context);
    assert_eq!(err_context.response().status(), 500);
    assert_eq!(
        err_context.response().body(),
        &ResponseBody::Text("error".to_string())
    );
}

#[test]
fn explicit_status_overrides_default() {
    let mut context = make_context("GET", "/status");
    Status::new(201, "created").into_response(&mut context);

    assert_eq!(context.response().status(), 201);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("created".to_string())
    );
}

#[test]
fn endpoint_execution_applies_into_response() {
    let mut context = make_context("GET", "/result");
    let metadata = EndpointMetadata::new("GET", "/result");
    let endpoint = Endpoint::new(
        EndpointKind::Http(HttpEndpointHandler::new(ValueEndpoint)),
        metadata,
    );
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);

    assert!(result.is_ok());
    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Text("from-endpoint".to_string())
    );
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("text/plain; charset=utf-8")
    );
}

struct Status<T> {
    status: u16,
    inner: T,
}

impl<T> Status<T> {
    fn new(status: u16, inner: T) -> Self {
        Self { status, inner }
    }
}

impl<T> IntoResponse for Status<T>
where
    T: IntoResponse,
{
    fn into_response(self, ctx: &mut HttpContext) {
        self.inner.into_response(ctx);
        ctx.response_mut().set_status(self.status);
    }
}

struct ValueEndpoint;

impl HttpHandler for ValueEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("from-endpoint"))
    }
}

fn make_context(method: &str, path: &str) -> HttpContext {
    let request = HttpRequest::new(method, path);
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    HttpContext::new(request, services, config)
}
