use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::request_body::RequestBody;
use nimble_web::http::request_body::RequestBodyStream;
use nimble_web::http::request_body::RequestBodyStreamHandle;
use nimble_web::http::response::HttpResponse;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::validation::ValidationError;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[test]
fn http_request_creation_headers_body() {
    let mut request = HttpRequest::new("GET", "/health");
    assert_eq!(request.method(), "GET");
    assert_eq!(request.path(), "/health");

    request.headers_mut().insert("x-test", "true");
    assert_eq!(request.headers().get("x-test"), Some("true"));

    request.set_body(RequestBody::Text("hello".to_string()));
    assert_eq!(request.body(), &RequestBody::Text("hello".to_string()));
}

#[test]
fn http_response_defaults_and_mutation() {
    let mut response = HttpResponse::default();
    assert_eq!(response.status(), 200);

    response.set_status(201);
    response.headers_mut().insert("content-type", "text/plain");
    response.set_body(ResponseBody::Bytes(vec![1, 2, 3]));

    assert_eq!(response.status(), 201);
    assert_eq!(response.headers().get("content-type"), Some("text/plain"));
    assert_eq!(response.body(), &ResponseBody::Bytes(vec![1, 2, 3]));
}

#[test]
fn http_context_basics() {
    let request = HttpRequest::new("POST", "/items");
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    let mut context = HttpContext::new(request, services, config);

    assert_eq!(context.response().status(), 404);

    context.insert::<u64>(42);
    assert_eq!(context.get::<u64>(), Some(&42));

    assert!(context.services().resolve::<u8>().is_none());

    assert!(context.config().get("missing.key").is_none());
}

#[test]
fn http_response_into_body_returns_owned_body() {
    let mut response = HttpResponse::new();
    response.set_body(ResponseBody::Text("owned".to_string()));
    let body = response.into_body();
    assert_eq!(body, ResponseBody::Text("owned".to_string()));
}

#[derive(Debug, Deserialize)]
struct Payload {
    name: String,
}

struct ChunkedStream {
    chunks: Vec<Vec<u8>>,
    index: usize,
}

impl ChunkedStream {
    fn new(chunks: Vec<Vec<u8>>) -> Self {
        Self { chunks, index: 0 }
    }
}

impl RequestBodyStream for ChunkedStream {
    fn read_chunk(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        if self.index >= self.chunks.len() {
            return Ok(None);
        }
        let chunk = self.chunks[self.index].clone();
        self.index += 1;
        Ok(Some(chunk))
    }
}

#[test]
fn http_context_reads_json_text_body() {
    let mut request = HttpRequest::new("POST", "/payload");
    request.set_body(RequestBody::Text("{\"name\":\"nimble\"}".to_string()));
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    let context = HttpContext::new(request, services, config);

    let payload: Payload = context.read_json().expect("payload");
    assert_eq!(payload.name, "nimble");
}

#[test]
fn http_context_reads_json_bytes_body() {
    let mut request = HttpRequest::new("POST", "/payload");
    request.set_body(RequestBody::Bytes(b"{\"name\":\"bytes\"}".to_vec()));
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    let context = HttpContext::new(request, services, config);

    let payload: Payload = context.read_body_as().expect("payload");
    assert_eq!(payload.name, "bytes");
}

#[test]
fn http_context_reads_json_stream_body() {
    let mut request = HttpRequest::new("POST", "/payload");
    let stream = ChunkedStream::new(vec![b"{\"name\":".to_vec(), b"\"stream\"}".to_vec()]);
    let handle: RequestBodyStreamHandle = Arc::new(Mutex::new(stream));
    request.set_body(RequestBody::Stream(handle));
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    let context = HttpContext::new(request, services, config);

    let payload: Payload = context.read_json().expect("payload");
    assert_eq!(payload.name, "stream");
}

#[test]
fn http_context_empty_body_returns_validation_error() {
    let request = HttpRequest::new("POST", "/payload");
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    let context = HttpContext::new(request, services, config);

    let error: ValidationError = context
        .read_json::<Payload>()
        .expect_err("expected error");
    assert_eq!(error.message(), "empty request body");
}
