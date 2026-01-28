use crate::endpoint::endpoint::{Endpoint, EndpointFuture};
use crate::endpoint::metadata::EndpointMetadata;
use crate::endpoint::ws_endpoint_handler::WsEndpointHandler;
use crate::http::context::HttpContext;

#[derive(Clone)]
pub struct WsEndpoint {
    _handler: WsEndpointHandler,
    metadata: EndpointMetadata,
}

impl WsEndpoint {
    pub fn new(handler: WsEndpointHandler, metadata: EndpointMetadata) -> Self {
        Self {
            _handler: handler,
            metadata,
        }
    }
}

impl Endpoint for WsEndpoint {
    fn metadata(&self) -> &EndpointMetadata {
        &self.metadata
    }

    fn invoke<'a>(&'a self, _context: &'a mut HttpContext) -> EndpointFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}
