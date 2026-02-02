use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_endpoint::HttpEndpoint;
use nimble_web::endpoint::http_endpoint_handler::HttpEndpointHandler;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::{IntoResponse, ResponseValue};
use nimble_web::result::{FileResponse, HttpError, Json};

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
fn json_response_error_sets_500() {
    struct BadJson;

    impl serde::Serialize for BadJson {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("boom"))
        }
    }

    let mut context = make_context("GET", "/json-error");
    Json(BadJson).into_response(&mut context);

    assert_eq!(context.response().status(), 500);
    assert_eq!(context.response().body(), &ResponseBody::Empty);
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
fn http_error_exposes_status_and_message() {
    let error = HttpError::new(418, "teapot");
    assert_eq!(error.status(), 418);
    assert_eq!(error.message(), "teapot");
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
    let endpoint = Arc::new(HttpEndpoint::new(
        HttpEndpointHandler::new(ValueEndpoint),
        metadata,
    ));
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

#[test]
fn file_response_uses_stream_body() {
    let mut context = make_context("GET", "/file");
    let temp_path = std::env::temp_dir().join(format!("nimble-web-stream-{}.txt", unique_suffix()));
    std::fs::write(&temp_path, b"stream me").expect("write temp file");

    FileResponse::from_path(&temp_path).into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert!(matches!(context.response().body(), ResponseBody::Stream(_)));
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("text/plain; charset=utf-8")
    );

    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn file_response_stream_reads_chunks() {
    let mut context = make_context("GET", "/file-read");
    let temp_path = std::env::temp_dir().join(format!("nimble-web-read-{}.txt", unique_suffix()));
    std::fs::write(&temp_path, b"chunked").expect("write temp file");

    FileResponse::from_path(&temp_path).into_response(&mut context);

    let body = std::mem::take(context.response_mut()).into_body();
    match body {
        ResponseBody::Stream(mut stream) => {
            let first = stream.read_chunk().expect("read").expect("chunk");
            assert_eq!(first, b"chunked".to_vec());
            let second = stream.read_chunk().expect("read");
            assert!(second.is_none());
        }
        other => panic!("expected stream body, got {:?}", other),
    }

    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn file_response_from_bytes_uses_default_content_type() {
    let mut context = make_context("GET", "/file-bytes");
    FileResponse::from_bytes(b"payload".to_vec()).into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &ResponseBody::Bytes(b"payload".to_vec())
    );
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("application/octet-stream")
    );
}

#[test]
fn file_response_respects_custom_content_type_and_filename() {
    let mut context = make_context("GET", "/file-custom");
    FileResponse::from_bytes(b"image".to_vec())
        .with_content_type("image/custom")
        .with_filename("custom.bin")
        .into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().headers().get("content-type"),
        Some("image/custom")
    );
    assert_eq!(
        context.response().headers().get("content-disposition"),
        Some("attachment; filename=\"custom.bin\"")
    );
}

#[test]
fn file_response_missing_path_sets_not_found() {
    let mut context = make_context("GET", "/missing");
    let missing = std::env::temp_dir().join(format!("nimble-web-missing-{}.txt", unique_suffix()));

    FileResponse::from_path(&missing).into_response(&mut context);

    assert_eq!(context.response().status(), 404);
    assert_eq!(context.response().body(), &ResponseBody::Empty);
    assert_eq!(context.response().headers().get("content-type"), None);
}

#[test]
fn file_response_sets_content_length() {
    let mut context = make_context("GET", "/file-len");
    let temp_path = std::env::temp_dir().join(format!("nimble-web-len-{}.txt", unique_suffix()));
    let data = b"some data";
    std::fs::write(&temp_path, data).expect("write temp file");

    FileResponse::from_path(&temp_path).into_response(&mut context);

    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().headers().get("content-length"),
        Some(data.len().to_string().as_str())
    );

    let _ = std::fs::remove_file(&temp_path);
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

#[async_trait]
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

fn unique_suffix() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos()
}
