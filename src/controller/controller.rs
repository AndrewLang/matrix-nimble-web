use crate::controller::registry::ControllerRegistry;

pub trait Controller: Send + Sync + 'static {
    fn register(registry: &mut ControllerRegistry);
}
