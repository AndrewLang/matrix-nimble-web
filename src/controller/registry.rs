use crate::controller::controller::Controller;
use crate::endpoint::endpoint::Endpoint;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use crate::endpoint::metadata::EndpointMetadata;
use crate::openapi::handler::OpenApiHandler;
use crate::openapi::registry::{OpenApiOperationMetadata, OpenApiRegistry};
use crate::routing::route::Route;
use crate::security::policy::Policy;

pub struct ControllerRegistry {
    routes: Vec<Route>,
    endpoints: Vec<Endpoint>,
    openapi_registry: OpenApiRegistry,
}

pub struct ControllerActionBuilder<'a> {
    registry: &'a mut ControllerRegistry,
    method: &'static str,
    path: String,
    endpoint: Endpoint,
    openapi: OpenApiOperationMetadata,
}

impl<'a> ControllerActionBuilder<'a> {
    pub fn validate<T>(mut self, validator: T) -> Self
    where
        T: crate::validation::AnyValidator + 'static,
    {
        let metadata = self.endpoint.metadata().clone().add_validator(validator);
        self.endpoint = Endpoint::new(self.endpoint.kind().clone(), metadata);
        self
    }

    pub fn body<T>(mut self) -> Self
    where
        T: crate::openapi::OpenApiSchema,
    {
        let schema_ref = self.registry.openapi_registry.register_schema::<T>();
        self.openapi.request_body = Some(schema_ref);
        self
    }

    pub fn param<T>(mut self, name: &str) -> Self
    where
        T: crate::openapi::OpenApiSchema,
    {
        let schema_ref = self.registry.openapi_registry.register_schema::<T>();
        self.openapi
            .path_params
            .insert(name.to_string(), schema_ref);
        self
    }

    pub fn query<T>(mut self, name: &str) -> Self
    where
        T: crate::openapi::OpenApiSchema,
    {
        let schema_ref = self.registry.openapi_registry.register_schema::<T>();
        self.openapi
            .query_params
            .insert(name.to_string(), schema_ref);
        self
    }

    pub fn responds<T>(mut self, status: u16) -> Self
    where
        T: crate::openapi::OpenApiSchema,
    {
        let schema_ref = self.registry.openapi_registry.register_schema::<T>();
        self.openapi.responses.insert(status, schema_ref);
        self
    }

    pub fn summary(mut self, value: &str) -> Self {
        self.openapi.summary = Some(value.to_string());
        self
    }

    pub fn description(mut self, value: &str) -> Self {
        self.openapi.description = Some(value.to_string());
        self
    }

    pub fn tag(mut self, value: &str) -> Self {
        self.openapi.tags.push(value.to_string());
        self
    }

    pub fn register(self) {
        let route = Route::new(self.method, &self.path);
        self.registry
            .add_route_with_openapi(route, self.endpoint, self.openapi);
    }
}

impl ControllerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<C: Controller>(&mut self) {
        C::register(self);
    }

    pub fn add<H>(&mut self, method: &str, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let route = Route::new(method, path);
        let metadata = EndpointMetadata::new(method, path);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        self.add_route(route, endpoint);
    }

    pub fn post<H>(&mut self, path: &str, handler: H) -> ControllerActionBuilder<'_>
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.action("POST", path, handler)
    }

    pub fn get<H>(&mut self, path: &str, handler: H) -> ControllerActionBuilder<'_>
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.action("GET", path, handler)
    }

    fn action<H>(
        &mut self,
        method: &'static str,
        path: &str,
        handler: H,
    ) -> ControllerActionBuilder<'_>
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let metadata = EndpointMetadata::new(method, path);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        ControllerActionBuilder {
            registry: self,
            method,
            path: path.to_string(),
            endpoint,
            openapi: OpenApiOperationMetadata::default(),
        }
    }

    pub fn add_with_policy<H>(&mut self, method: &str, path: &str, handler: H, policy: Policy)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let requires_auth = matches!(policy, Policy::Authenticated);
        let route = Route::new(method, path);
        let metadata = EndpointMetadata::new(method, path).require_policy(policy);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        let mut openapi = OpenApiOperationMetadata::new();
        if requires_auth {
            openapi.requires_auth = true;
        }
        self.add_route_with_openapi(route, endpoint, openapi);
    }

    pub fn add_route(&mut self, route: Route, endpoint: Endpoint) {
        self.add_route_with_openapi(
            route,
            endpoint,
            OpenApiOperationMetadata::default(),
        );
    }

    fn add_route_with_openapi(
        &mut self,
        route: Route,
        endpoint: Endpoint,
        openapi: OpenApiOperationMetadata,
    ) {
        self.openapi_registry
            .add_with_metadata(route.method(), route.path(), openapi);
        self.routes.push(route);
        self.endpoints.push(endpoint);
    }

    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    pub fn find_endpoint(&self, route: &Route) -> Option<Endpoint> {
        self.routes
            .iter()
            .position(|candidate| candidate == route)
            .and_then(|index| self.endpoints.get(index).cloned())
    }

    pub fn openapi_registry(&self) -> OpenApiRegistry {
        self.openapi_registry.clone()
    }

    pub fn merge_openapi_registry(&mut self, registry: OpenApiRegistry) {
        self.openapi_registry.merge(registry);
    }

    pub fn ensure_openapi_endpoint(&mut self) -> bool {
        if self
            .routes
            .iter()
            .any(|route| route.method() == "GET" && route.path() == "/openapi.json")
        {
            return false;
        }

        let handler = OpenApiHandler::new(self.openapi_registry.clone());
        let metadata = EndpointMetadata::new("GET", "/openapi.json");
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        let route = Route::new("GET", "/openapi.json");
        self.add_route_with_openapi(route, endpoint, OpenApiOperationMetadata::default());
        true
    }
}

impl Default for ControllerRegistry {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            endpoints: Vec::new(),
            openapi_registry: OpenApiRegistry::new(),
        }
    }
}
