use crate::config::{ConfigBuilder, Configuration};
use crate::di::{ServiceContainer, ServiceProvider};
use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;

pub struct HttpContextBuilder {
    request: HttpRequest,
    response: HttpResponse,
    services: ServiceProvider,
    config: Configuration,
}

impl HttpContextBuilder {
    pub fn new() -> Self {
        Self {
            request: HttpRequest::new("GET", "/"),
            response: HttpResponse::new(),
            services: ServiceContainer::new().build(),
            config: ConfigBuilder::new().build(),
        }
    }

    pub fn request(mut self, request: HttpRequest) -> Self {
        self.request = request;
        self
    }

    pub fn response(mut self, response: HttpResponse) -> Self {
        self.response = response;
        self
    }

    pub fn services(mut self, services: ServiceProvider) -> Self {
        self.services = services;
        self
    }

    pub fn config(mut self, config: Configuration) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> HttpContext {
        let mut context = HttpContext::new(self.request, self.services, self.config);
        *context.response_mut() = self.response;
        context
    }
}
