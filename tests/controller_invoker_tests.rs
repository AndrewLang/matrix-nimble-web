use std::sync::Arc;

use nimble_web::config::ConfigBuilder;
use nimble_web::controller::invoker::{ControllerInvoker, ControllerInvokerMiddleware};
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::endpoint::Endpoint;
use nimble_web::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::middleware::endpoint_exec::EndpointExecutionMiddleware;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};
use nimble_web::result::into_response::ResponseValue;
use nimble_web::routing::route::Route;
use nimble_web::routing::route_data::RouteData;

#[test]
fn controller_invoker_handler_builds_endpoint_handler() {
    #[derive(Default)]
    struct Controller {
        name: &'static str,
    }

    let controller = Arc::new(Controller { name: "nimble" });
    let invoker = ControllerInvoker::new(controller);
    let handler = invoker.handler(|ctrl, _ctx| format!("hello {}", ctrl.name));

    let mut context = HttpContext::new(
        HttpRequest::new("GET", "/hi"),
        ServiceContainer::new().build(),
        ConfigBuilder::new().build(),
    );
    let endpoint = Endpoint::new(
        EndpointKind::Http(handler),
        EndpointMetadata::new("GET", "/hi"),
    );
    context.set_endpoint(endpoint);

    let mut pipeline = Pipeline::new();
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);
    assert!(result.is_ok());
    assert_eq!(
        context.response().body(),
        &nimble_web::http::response_body::ResponseBody::Text("hello nimble".to_string())
    );
}

#[test]
fn controller_invoker_middleware_sets_endpoint_from_registry() {
    let mut registry = ControllerRegistry::new();
    let metadata = EndpointMetadata::new("GET", "/items");
    let endpoint = Endpoint::new(
        EndpointKind::Http(HttpEndpointHandler::new(NullHandler)),
        metadata,
    );
    let route = Route::new("GET", "/items");
    registry.add_route(route.clone(), endpoint.clone());

    let middleware = ControllerInvokerMiddleware::new(Arc::new(registry));
    let mut pipeline = Pipeline::new();
    pipeline.add(middleware);

    let mut context = HttpContext::new(
        HttpRequest::new("GET", "/items"),
        ServiceContainer::new().build(),
        ConfigBuilder::new().build(),
    );
    let route_data = RouteData::new(route, std::collections::HashMap::new());
    context.set_route(route_data);

    let result = pipeline.run(&mut context);
    assert!(result.is_ok());
    assert!(context.endpoint().is_some());
}

struct NullHandler;

#[allow(async_fn_in_trait)]
impl nimble_web::endpoint::http_handler::HttpHandler for NullHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new(""))
    }
}
