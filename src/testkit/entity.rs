use std::sync::Arc;

use crate::app::builder::AppBuilder;
use crate::entity::entity::Entity;
use crate::entity::registry::EntityRegistry;

pub trait EntityTestkit {
    fn add_entity<T: Entity>(&mut self) -> &mut Self;
    fn entity_registry(&self) -> Arc<EntityRegistry>;
    fn assert_has_entity(&self, name: &str);
}

impl EntityTestkit for AppBuilder {
    fn add_entity<T: Entity>(&mut self) -> &mut Self {
        AppBuilder::add_entity::<T>(self)
    }

    fn entity_registry(&self) -> Arc<EntityRegistry> {
        let registry = Arc::new(self.entity_registry_clone());
        registry
    }

    fn assert_has_entity(&self, name: &str) {
        let registry = self.entity_registry();
        assert!(
            registry.get(name).is_some(),
            "expected entity registry to contain {}",
            name
        );
    }
}

pub trait EntityRegistryAssertions {
    fn assert_entity(&self, name: &str);
}

impl EntityRegistryAssertions for Arc<EntityRegistry> {
    fn assert_entity(&self, name: &str) {
        assert!(
            self.get(name).is_some(),
            "expected entity registry to contain {}",
            name
        );
    }
}
