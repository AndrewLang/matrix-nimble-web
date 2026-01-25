use crate::endpoint::http_handler::HttpHandler;
use crate::openapi::generator::OpenApiGenerator;
use crate::openapi::registry::OpenApiRegistry;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;
use crate::result::Json;

pub struct OpenApiHandler {
    registry: OpenApiRegistry,
}

impl OpenApiHandler {
    pub fn new(registry: OpenApiRegistry) -> Self {
        Self { registry }
    }
}

impl HttpHandler for OpenApiHandler {
    async fn invoke(
        &self,
        _context: &mut crate::http::context::HttpContext,
    ) -> Result<ResponseValue, PipelineError> {
        let document = OpenApiGenerator::generate(&self.registry);
        Ok(ResponseValue::new(Json(document)))
    }
}
