use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::http::request_body::RequestBody;
use nimble_web::http::response::HttpResponse;
use nimble_web::http::response_body::ResponseBody;

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
