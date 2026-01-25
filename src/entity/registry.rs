use std::collections::HashMap;
use std::sync::RwLock;

use crate::entity::entity::Entity;
use crate::entity::metadata::EntityMetadata;

pub struct EntityRegistry {
    entries: RwLock<HashMap<&'static str, &'static EntityMetadata>>,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<T: Entity>(&mut self) {
        let metadata = EntityMetadata::of::<T>();
        let leaked = Box::leak(Box::new(metadata));
        let mut guard = self.entries.write().expect("entity registry write");
        guard.insert(leaked.name(), leaked);
    }

    pub fn get(&self, name: &str) -> Option<&EntityMetadata> {
        let guard = self.entries.read().expect("entity registry read");
        guard.get(name).copied()
    }

    pub(crate) fn from_registry(other: &EntityRegistry) -> Self {
        let registry = EntityRegistry::new();
        if let Ok(guard) = other.entries.read() {
            for (name, metadata) in guard.iter() {
                let leaked = Box::leak(Box::new((*metadata).clone()));
                registry
                    .entries
                    .write()
                    .expect("entity registry write")
                    .insert(*name, leaked);
            }
        }
        registry
    }
}
