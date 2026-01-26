use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::middleware::validation::ValidationMiddleware;
use nimble_web::pipeline::middleware::Middleware;
use nimble_web::pipeline::next::Next;
use nimble_web::pipeline::pipeline::{Pipeline, PipelineError};

struct MarkerMiddleware;

impl Middleware for MarkerMiddleware {
    async fn handle(&self, context: &mut HttpContext, _next: Next<'_>) -> Result<(), PipelineError> {
        context.insert::<u32>(7);
        Ok(())
    }
}

#[test]
fn passthrough_validation_middleware_allows_next() {
    let request = HttpRequest::new("GET", "/");
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    let mut context = HttpContext::new(request, services, config);

    let mut pipeline = Pipeline::new();
    pipeline.add(ValidationMiddleware::new());
    pipeline.add(MarkerMiddleware);

    let result = pipeline.run(&mut context);
    assert!(result.is_ok());
    assert_eq!(context.get::<u32>(), Some(&7));
}
