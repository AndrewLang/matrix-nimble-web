use crate::endpoint::endpoint::{Endpoint, EndpointFuture};
use crate::endpoint::http_endpoint_handler::HttpEndpointHandler;
use crate::endpoint::metadata::EndpointMetadata;
use crate::http::context::HttpContext;

#[derive(Clone)]
pub struct HttpEndpoint {
    handler: HttpEndpointHandler,
    metadata: EndpointMetadata,
}

impl HttpEndpoint {
    pub fn new(handler: HttpEndpointHandler, metadata: EndpointMetadata) -> Self {
        Self { handler, metadata }
    }
}

impl Endpoint for HttpEndpoint {
    fn metadata(&self) -> &EndpointMetadata {
        &self.metadata
    }

    fn invoke<'a>(&'a self, context: &'a mut HttpContext) -> EndpointFuture<'a> {
        let handler = self.handler.clone();
        Box::pin(async move {
            let value = handler.invoke(context).await?;
            value.apply(context);
            Ok(())
        })
    }
}
