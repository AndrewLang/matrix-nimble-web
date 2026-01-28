use std::sync::Arc;

use crate::app::builder::AppBuilder;
use crate::entity::entity::Entity;
use crate::entity::registry::EntityRegistry;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::str::FromStr;

pub trait EntityTestkit {
    fn add_entity<E>(&mut self) -> &mut Self
    where
        E: Entity + Serialize + DeserializeOwned + 'static,
        E::Id: FromStr + Send + Sync + 'static;
    fn entity_registry(&self) -> Arc<EntityRegistry>;
    fn assert_has_entity(&self, name: &str);
}

impl EntityTestkit for AppBuilder {
    fn add_entity<E>(&mut self) -> &mut Self
    where
        E: Entity + Serialize + DeserializeOwned + 'static,
        E::Id: FromStr + Send + Sync + 'static,
    {
        AppBuilder::use_entity::<E>(self)
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
