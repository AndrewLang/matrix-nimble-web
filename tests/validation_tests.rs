use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request_body::RequestBody;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::testkit::app::TestApp;
use nimble_web::testkit::request::HttpRequestBuilder;
use nimble_web::validation::ValidationMiddleware;
use nimble_web::validation::{ContextValidator, ValidationError};

#[derive(Debug, Clone)]
struct FakeDto {
    name: String,
}

impl FakeDto {
    fn from_body(body: &RequestBody) -> Option<Self> {
        match body {
            RequestBody::Text(text) if text == "valid" => Some(Self {
                name: "valid".to_string(),
            }),
            _ => None,
        }
    }
}

impl FakeDto {
    fn name(&self) -> &str {
        &self.name
    }
}

use async_trait::async_trait;

struct TestEndpoint;

#[async_trait(?Send)]
impl HttpHandler for TestEndpoint {
    async fn invoke(&self, _context: &HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

fn validator_passes() -> ContextValidator {
    ContextValidator::new(|context| {
        let body = context.request().body();
        let dto = FakeDto::from_body(body).ok_or_else(|| ValidationError::new("invalid"))?;
        let _ = dto.name();
        Ok(())
    })
}

fn validator_fails() -> ContextValidator {
    ContextValidator::new(|_context| Err(ValidationError::new("invalid")))
}

use nimble_web::endpoint::route::EndpointRoute;

struct ValidController;

impl Controller for ValidController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::post("/validate", TestEndpoint)
            .validate(validator_passes())
            .build()]
    }
}

struct InvalidController;

impl Controller for InvalidController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::post("/validate", TestEndpoint)
            .validate(validator_fails())
            .build()]
    }
}

struct NoValidatorController;

impl Controller for NoValidatorController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::post("/validate", TestEndpoint).build()]
    }
}

struct MultiValidatorController;

impl Controller for MultiValidatorController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::post("/validate", TestEndpoint)
            .validate(validator_passes())
            .validate(validator_fails())
            .validate(validator_passes())
            .build()]
    }
}

#[test]
fn validation_passes() {
    let request = HttpRequestBuilder::post("/validate").body("valid").build();
    let response = TestApp::new()
        .use_middleware(ValidationMiddleware::new())
        .add_controller::<ValidController>()
        .run(request);

    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), &ResponseBody::Text("ok".to_string()));
}

#[test]
fn validation_fails() {
    let request = HttpRequestBuilder::post("/validate")
        .body("invalid")
        .build();
    let response = TestApp::new()
        .use_middleware(ValidationMiddleware::new())
        .add_controller::<InvalidController>()
        .run(request);

    assert_eq!(response.status(), 400);
    assert!(matches!(response.body(), ResponseBody::Text(text) if text.contains("invalid")));
}

#[test]
fn no_validator_attached() {
    let request = HttpRequestBuilder::post("/validate").body("valid").build();
    let response = TestApp::new()
        .use_middleware(ValidationMiddleware::new())
        .add_controller::<NoValidatorController>()
        .run(request);

    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), &ResponseBody::Text("ok".to_string()));
}

#[test]
fn multiple_validators_short_circuit() {
    let request = HttpRequestBuilder::post("/validate").body("valid").build();
    let response = TestApp::new()
        .use_middleware(ValidationMiddleware::new())
        .add_controller::<MultiValidatorController>()
        .run(request);

    assert_eq!(response.status(), 400);
}

#[test]
fn validation_does_not_require_auth() {
    let request = HttpRequestBuilder::post("/validate").body("valid").build();
    let response = TestApp::new()
        .use_middleware(ValidationMiddleware::new())
        .add_controller::<ValidController>()
        .run(request);

    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), &ResponseBody::Text("ok".to_string()));
}
