use crate::app::application::Application;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::pipeline::Pipeline;

pub struct AppBuilder {
    pipeline: Pipeline,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
        }
    }

    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.pipeline.add(middleware);
        self
    }

    pub fn build(self) -> Application {
        Application::new(self.pipeline)
    }
}
