use serde::de::DeserializeOwned;
use std::any::type_name;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Configuration;
use crate::di::ServiceProvider;
use crate::endpoint::endpoint::Endpoint;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;
use crate::routing::route_data::RouteData;
use crate::validation::ValidationError;

pub struct HttpContext {
    request: HttpRequest,
    response: HttpResponse,
    services: ServiceProvider,
    config: Arc<Configuration>,
    items: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    route: Option<RouteData>,
    endpoint: Option<Arc<dyn Endpoint>>,
    response_value: Option<ResponseValue>,
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
            route: None,
            endpoint: None,
            response_value: None,
        }
    }

    pub fn request(&self) -> &HttpRequest {
        &self.request
    }

    pub fn request_mut(&mut self) -> &mut HttpRequest {
        &mut self.request
    }

    pub fn read_json<T>(&self) -> Result<T, ValidationError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.read_body_as()
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

    pub fn route(&self) -> Option<&RouteData> {
        self.route.as_ref()
    }

    pub fn set_route(&mut self, route: RouteData) {
        self.route = Some(route);
    }

    pub fn endpoint(&self) -> Option<&Arc<dyn Endpoint>> {
        self.endpoint.as_ref()
    }

    pub fn set_endpoint(&mut self, endpoint: Arc<dyn Endpoint>) {
        self.endpoint = Some(endpoint);
    }

    pub fn set_response_value<T>(&mut self, value: T)
    where
        T: crate::result::IntoResponse + Send + Sync + 'static,
    {
        self.response_value = Some(ResponseValue::new(value));
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

    pub fn read_body_as<T>(&self) -> Result<T, ValidationError>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.request.body() {
            crate::http::request_body::RequestBody::Text(text) => {
                serde_json::from_str(text).map_err(|err| ValidationError::new(&err.to_string()))
            }
            crate::http::request_body::RequestBody::Bytes(bytes) => {
                let text = std::str::from_utf8(bytes)
                    .map_err(|err| ValidationError::new(&err.to_string()))?;
                serde_json::from_str(text).map_err(|err| {
                    log::error!("❌ JSON deserialization failed: {}. Body: {}", err, text);
                    ValidationError::new(&err.to_string())
                })
            }
            crate::http::request_body::RequestBody::Stream(stream) => {
                let mut collected = Vec::new();
                let mut guard = stream
                    .lock()
                    .map_err(|_| ValidationError::new("request body stream lock error"))?;
                loop {
                    match guard
                        .read_chunk()
                        .map_err(|err| ValidationError::new(&err.to_string()))?
                    {
                        Some(chunk) => collected.extend_from_slice(&chunk),
                        None => break,
                    }
                }
                let text = std::str::from_utf8(&collected)
                    .map_err(|err| ValidationError::new(&err.to_string()))?;
                serde_json::from_str(text).map_err(|err| {
                    log::error!("❌ JSON deserialization failed: {}. Body: {}", err, text);
                    ValidationError::new(&err.to_string())
                })
            }
            crate::http::request_body::RequestBody::Empty => {
                Err(ValidationError::new("empty request body"))
            }
        }
    }
}

impl HttpContext {
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, PipelineError> {
        self.read_json()
            .map_err(|e| PipelineError::message(&e.message()))
    }

    pub fn service<T>(&self) -> Result<Arc<T>, PipelineError>
    where
        T: Send + Sync + 'static,
    {
        self.services().resolve::<T>().ok_or_else(|| {
            PipelineError::message(&format!(
                "Service `{}` is not registered",
                Self::short_type_name::<T>()
            ))
        })
    }

    fn short_type_name<T>() -> &'static str {
        type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap_or(type_name::<T>())
    }
}
