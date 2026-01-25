use std::any::{type_name, TypeId};
use std::collections::HashMap;

use crate::openapi::model::{Schema, SchemaRef};

pub trait OpenApiSchema {
    fn schema() -> Schema;

    fn schema_name() -> String
    where
        Self: Sized,
    {
        type_name_short::<Self>()
    }
}

impl OpenApiSchema for String {
    fn schema() -> Schema {
        Schema::string()
    }
}

impl OpenApiSchema for i32 {
    fn schema() -> Schema {
        Schema::integer_with_format("int32")
    }
}

impl OpenApiSchema for i64 {
    fn schema() -> Schema {
        Schema::integer_with_format("int64")
    }
}

#[derive(Clone, Debug, Default)]
pub struct SchemaRegistry {
    schemas: HashMap<String, Schema>,
    overrides: HashMap<TypeId, String>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            overrides: HashMap::new(),
        }
    }

    pub fn register<T: OpenApiSchema + 'static>(&mut self) -> SchemaRef {
        let name = self
            .overrides
            .get(&TypeId::of::<T>())
            .cloned()
            .unwrap_or_else(T::schema_name);
        self.schemas.entry(name.clone()).or_insert_with(T::schema);
        SchemaRef {
            ref_path: format!("#/components/schemas/{}", name),
        }
    }

    pub fn schemas(&self) -> &HashMap<String, Schema> {
        &self.schemas
    }

    pub fn merge(&mut self, other: &SchemaRegistry) {
        for (name, schema) in other.schemas() {
            self.schemas
                .entry(name.clone())
                .or_insert_with(|| schema.clone());
        }
        for (type_id, name) in &other.overrides {
            self.overrides
                .entry(*type_id)
                .or_insert_with(|| name.clone());
        }
    }

    pub fn set_type_name<T: 'static>(&mut self, name: &str) {
        self.overrides.insert(TypeId::of::<T>(), name.to_string());
    }
}

fn type_name_short<T>() -> String {
    let full = type_name::<T>();
    full.rsplit("::").next().unwrap_or(full).to_string()
}
