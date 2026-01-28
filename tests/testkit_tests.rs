use nimble_web::config::ConfigBuilder;
use nimble_web::controller::controller::Controller;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::auth::AuthenticationMiddleware;
use nimble_web::testkit;
use nimble_web::testkit::auth::{assert_authenticated, WithAuth};
use nimble_web::testkit::response::ResponseAssertions;

struct TestEndpoint;

impl HttpHandler for TestEndpoint {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

struct TestController;

use nimble_web::controller::registry::EndpointRoute;

impl Controller for TestController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::get("/controller", TestEndpoint).build()]
    }
}

#[test]
fn build_http_request() {
    let request = testkit::request::HttpRequestBuilder::get("/test")
        .header("x-test", "1")
        .body("hello")
        .build();

    assert_eq!(request.method(), "GET");
    assert_eq!(request.path(), "/test");
    assert_eq!(request.headers().get("x-test"), Some("1"));
    assert!(
        matches!(request.body(), nimble_web::http::request_body::RequestBody::Text(text) if text == "hello")
    );
}

#[test]
fn build_http_context() {
    let request = HttpRequest::new("GET", "/ctx");
    let context = testkit::context::HttpContextBuilder::new()
        .request(request)
        .services(ServiceContainer::new().build())
        .config(ConfigBuilder::new().build())
        .build();

    assert_eq!(context.request().path(), "/ctx");
    assert_eq!(context.response().status(), 200);
}

#[test]
fn run_pipeline_end_to_end() {
    let response = testkit::app::TestApp::new()
        .add_controller::<TestController>()
        .run(HttpRequest::new("GET", "/controller"));

    response.assert_status(200);
    response.assert_body("ok");
}

#[test]
fn test_controller_easily() {
    let response = testkit::app::TestApp::new()
        .add_controller::<TestController>()
        .run(HttpRequest::new("GET", "/controller"));

    response.assert_status(200);
    response.assert_body("ok");
}

#[test]
fn auth_testing_support() {
    let request = testkit::request::HttpRequestBuilder::get("/secure")
        .build()
        .with_auth("test-user");
    let mut context = HttpContext::new(
        request,
        ServiceContainer::new().build(),
        ConfigBuilder::new().build(),
    );

    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());
    let _ = pipeline.run(&mut context);

    assert_authenticated(&context);
}
