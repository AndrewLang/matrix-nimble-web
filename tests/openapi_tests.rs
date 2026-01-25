use nimble_web::controller::controller::Controller;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::http::response_body::ResponseBody;
use nimble_web::openapi::model::OpenApiDocument;
use nimble_web::openapi::Schema;
use nimble_web::openapi::OpenApiSchema;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::testkit::app::TestApp;
use nimble_web::testkit::request::HttpRequestBuilder;

struct PhotosController;

impl Controller for PhotosController {
    fn register(registry: &mut ControllerRegistry) {
        registry.add("GET", "/photos", PhotosGet);
        registry.add("POST", "/photos", PhotosPost);
    }
}

struct PhotosGet;

impl HttpHandler for PhotosGet {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

struct PhotosPost;

impl HttpHandler for PhotosPost {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("created"))
    }
}

#[test]
fn openapi_endpoint_lists_routes_and_methods() {
    let request = HttpRequestBuilder::get("/openapi.json").build();
    let response = TestApp::new().add_controller::<PhotosController>().run(request);

    assert_eq!(response.status(), 200);

    let body = match response.body() {
        ResponseBody::Text(text) => text.clone(),
        ResponseBody::Bytes(bytes) => String::from_utf8(bytes.clone()).expect("utf-8 response"),
        other => panic!("unexpected response body: {:?}", other),
    };

    let document: OpenApiDocument =
        serde_json::from_str(&body).expect("openapi json payload");

    assert_eq!(document.openapi, "3.1.0");
    let photos = document.paths.get("/photos").expect("photos path");
    assert!(photos.operations.contains_key("get"));
    assert!(photos.operations.contains_key("post"));
}

#[test]
fn openapi_includes_summary_and_tags() {
    let request = HttpRequestBuilder::get("/openapi.json").build();
    let response = TestApp::new()
        .add_controller::<MetadataController>()
        .run(request);

    assert_eq!(response.status(), 200);

    let body = match response.body() {
        ResponseBody::Text(text) => text.clone(),
        ResponseBody::Bytes(bytes) => String::from_utf8(bytes.clone()).expect("utf-8 response"),
        other => panic!("unexpected response body: {:?}", other),
    };

    let document: OpenApiDocument =
        serde_json::from_str(&body).expect("openapi json payload");

    let path_item = document.paths.get("/widgets").expect("widgets path");
    let operation = path_item
        .operations
        .get("get")
        .expect("get operation");
    assert_eq!(operation.summary.as_deref(), Some("List widgets"));
    assert!(operation.tags.iter().any(|tag| tag == "widgets"));
}

#[test]
fn openapi_includes_request_and_response_schemas() {
    let request = HttpRequestBuilder::get("/openapi.json").build();
    let response = TestApp::new()
        .add_controller::<SchemaController>()
        .run(request);

    assert_eq!(response.status(), 200);

    let body = match response.body() {
        ResponseBody::Text(text) => text.clone(),
        ResponseBody::Bytes(bytes) => String::from_utf8(bytes.clone()).expect("utf-8 response"),
        other => panic!("unexpected response body: {:?}", other),
    };

    let document: OpenApiDocument =
        serde_json::from_str(&body).expect("openapi json payload");

    assert!(document.components.schemas.contains_key("CreateWidget"));
    assert!(document
        .components
        .schemas
        .contains_key("WidgetResponse"));

    let path_item = document.paths.get("/widgets").expect("widgets path");
    let operation = path_item
        .operations
        .get("post")
        .expect("post operation");

    let request_body = operation.request_body.as_ref().expect("request body");
    let request_schema = request_body
        .content
        .get("application/json")
        .and_then(|media| media.schema.as_ref())
        .expect("request schema");
    assert!(request_schema.ref_path.ends_with("/CreateWidget"));

    let response_schema = operation
        .responses
        .get("200")
        .and_then(|resp| resp.content.get("application/json"))
        .and_then(|media| media.schema.as_ref())
        .expect("response schema");
    assert!(response_schema.ref_path.ends_with("/WidgetResponse"));
}

#[test]
fn openapi_includes_parameters_and_auth() {
    let request = HttpRequestBuilder::get("/openapi.json").build();
    let response = TestApp::new()
        .add_controller::<ParamAuthController>()
        .run(request);

    assert_eq!(response.status(), 200);

    let body = match response.body() {
        ResponseBody::Text(text) => text.clone(),
        ResponseBody::Bytes(bytes) => String::from_utf8(bytes.clone()).expect("utf-8 response"),
        other => panic!("unexpected response body: {:?}", other),
    };

    let document: OpenApiDocument =
        serde_json::from_str(&body).expect("openapi json payload");

    let path_item = document.paths.get("/photos/{id}").expect("path params");
    let operation = path_item
        .operations
        .get("post")
        .expect("post operation");

    let id_param = operation
        .parameters
        .iter()
        .find(|param| param.name == "id" && param.location == "path")
        .expect("id param");
    assert!(id_param.required);
    assert!(id_param.schema.ref_path.ends_with("/i64"));

    let page_param = operation
        .parameters
        .iter()
        .find(|param| param.name == "page" && param.location == "query")
        .expect("page param");
    assert!(!page_param.required);
    assert!(page_param.schema.ref_path.ends_with("/i32"));

    let page_size_param = operation
        .parameters
        .iter()
        .find(|param| param.name == "pageSize" && param.location == "query")
        .expect("pageSize param");
    assert!(!page_size_param.required);
    assert!(page_size_param.schema.ref_path.ends_with("/i32"));

    let secure_item = document.paths.get("/secure").expect("secure path");
    let secure_op = secure_item
        .operations
        .get("get")
        .expect("get operation");
    assert!(secure_op
        .security
        .iter()
        .any(|entry| entry.contains_key("bearerAuth")));
    assert!(document
        .components
        .security_schemes
        .contains_key("bearerAuth"));
}

struct MetadataController;

impl Controller for MetadataController {
    fn register(registry: &mut ControllerRegistry) {
        registry
            .get("/widgets", WidgetsGet)
            .summary("List widgets")
            .tag("widgets")
            .register();
    }
}

struct WidgetsGet;

impl HttpHandler for WidgetsGet {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

struct SchemaController;

impl Controller for SchemaController {
    fn register(registry: &mut ControllerRegistry) {
        registry
            .post("/widgets", WidgetsPost)
            .body::<CreateWidget>()
            .responds::<WidgetResponse>(200)
            .register();
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct CreateWidget {
    name: String,
    quantity: i32,
}

#[derive(Debug)]
#[allow(dead_code)]
struct WidgetResponse {
    id: String,
}

impl OpenApiSchema for CreateWidget {
    fn schema() -> Schema {
        let mut properties = std::collections::HashMap::new();
        properties.insert("name".to_string(), Schema::string());
        properties.insert("quantity".to_string(), Schema::integer());
        Schema::object(properties, vec!["name".to_string(), "quantity".to_string()])
    }
}

impl OpenApiSchema for WidgetResponse {
    fn schema() -> Schema {
        let mut properties = std::collections::HashMap::new();
        properties.insert("id".to_string(), Schema::string());
        Schema::object(properties, vec!["id".to_string()])
    }
}

struct WidgetsPost;

impl HttpHandler for WidgetsPost {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("created"))
    }
}

struct ParamAuthController;

impl Controller for ParamAuthController {
    fn register(registry: &mut ControllerRegistry) {
        registry
            .post("/photos/{id}", ParamsPost)
            .param::<i64>("id")
            .query::<i32>("page")
            .query::<i32>("pageSize")
            .register();

        registry.add_with_policy("GET", "/secure", SecureGet, Policy::Authenticated);
    }
}

struct ParamsPost;

impl HttpHandler for ParamsPost {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("ok"))
    }
}

struct SecureGet;

impl HttpHandler for SecureGet {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("secure"))
    }
}
