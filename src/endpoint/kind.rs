use std::sync::Arc;

use crate::endpoint::http_handler::{DynHttpHandler, HttpHandler, HttpHandlerFuture};
use crate::http::context::HttpContext;

#[derive(Clone)]
pub struct HttpEndpointHandler {
    inner: Arc<dyn DynHttpHandler>,
}

impl HttpEndpointHandler {
    pub fn new<H>(handler: H) -> Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(handler),
        }
    }

    pub(crate) fn invoke<'a>(&'a self, ctx: &'a mut HttpContext) -> HttpHandlerFuture<'a> {
        self.inner.invoke(ctx)
    }
}

#[derive(Clone)]
pub struct WebSocketHandler;

#[derive(Clone)]
pub enum EndpointKind {
    Http(HttpEndpointHandler),
    WebSocket(WebSocketHandler),
}
