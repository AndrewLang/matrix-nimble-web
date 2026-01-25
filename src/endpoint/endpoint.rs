use crate::endpoint::kind::EndpointKind;
use crate::endpoint::metadata::EndpointMetadata;

#[derive(Clone)]
pub struct Endpoint {
    kind: EndpointKind,
    metadata: EndpointMetadata,
}

impl Endpoint {
    pub fn new(kind: EndpointKind, metadata: EndpointMetadata) -> Self {
        Self { kind, metadata }
    }

    pub fn kind(&self) -> &EndpointKind {
        &self.kind
    }

    pub fn metadata(&self) -> &EndpointMetadata {
        &self.metadata
    }
}
