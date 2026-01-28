use crate::endpoint::route::EndpointRoute;

pub trait Controller: Send + Sync + 'static {
    fn routes() -> Vec<EndpointRoute>;
}
