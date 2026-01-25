use crate::config::ConfigBuilder;
use crate::di::ServiceContainer;
use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::pipeline::pipeline::Pipeline;

pub struct Application {
    pipeline: Pipeline,
}

impl Application {
    pub(crate) fn new(pipeline: Pipeline) -> Self {
        Self { pipeline }
    }

    pub fn handle_http(&self, request: HttpRequest) -> HttpResponse {
        let services = ServiceContainer::new().build();
        let config = ConfigBuilder::new().build();
        let mut context = HttpContext::new(request, services, config);
        let _ = self.pipeline.run(&mut context);
        context.response().clone()
    }
}
