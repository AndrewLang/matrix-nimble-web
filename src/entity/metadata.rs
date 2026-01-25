use std::any::type_name;

use crate::entity::entity::Entity;

#[derive(Debug, Clone)]
pub struct EntityMetadata {
    name: &'static str,
    plural_name: &'static str,
    id_type_name: &'static str,
}

impl EntityMetadata {
    pub fn of<T: Entity>() -> Self {
        Self {
            name: T::name(),
            plural_name: T::plural_name(),
            id_type_name: type_name::<T::Id>(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn plural_name(&self) -> &'static str {
        self.plural_name
    }

    pub fn id_type_name(&self) -> &'static str {
        self.id_type_name
    }
}
