use crate::controller::registry::EndpointRoute;

pub trait Controller: Send + Sync + 'static {
    fn routes() -> Vec<EndpointRoute>;
}
