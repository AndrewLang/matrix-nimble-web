use crate::endpoint::endpoint::{Endpoint, EndpointFuture};
use crate::endpoint::metadata::EndpointMetadata;
use crate::endpoint::websocket_endpoint_handler::WebSocketEndpointHandler;
use crate::http::context::HttpContext;

#[derive(Clone)]
pub struct WebSocketEndpoint {
    _handler: WebSocketEndpointHandler,
    metadata: EndpointMetadata,
}

impl WebSocketEndpoint {
    pub fn new(handler: WebSocketEndpointHandler, metadata: EndpointMetadata) -> Self {
        Self {
            _handler: handler,
            metadata,
        }
    }
}

impl Endpoint for WebSocketEndpoint {
    fn metadata(&self) -> &EndpointMetadata {
        &self.metadata
    }

    fn invoke<'a>(&'a self, _context: &'a mut HttpContext) -> EndpointFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}
