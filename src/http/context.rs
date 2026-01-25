use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Configuration;
use crate::di::ServiceProvider;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;

pub struct HttpContext {
    request: HttpRequest,
    response: HttpResponse,
    services: ServiceProvider,
    config: Arc<Configuration>,
    items: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl HttpContext {
    pub fn new(request: HttpRequest, services: ServiceProvider, config: Configuration) -> Self {
        let mut response = HttpResponse::default();
        response.set_status(404);
        Self {
            request,
            response,
            services,
            config: Arc::new(config),
            items: HashMap::new(),
        }
    }

    pub fn request(&self) -> &HttpRequest {
        &self.request
    }

    pub fn response(&self) -> &HttpResponse {
        &self.response
    }

    pub fn response_mut(&mut self) -> &mut HttpResponse {
        &mut self.response
    }

    pub fn services(&self) -> &ServiceProvider {
        &self.services
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn insert<T>(&mut self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.items.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.items
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }
}
