use std::any::type_name;
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
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register<T: OpenApiSchema>(&mut self) -> SchemaRef {
        let name = T::schema_name();
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
            self.schemas.entry(name.clone()).or_insert_with(|| schema.clone());
        }
    }
}

fn type_name_short<T>() -> String {
    let full = type_name::<T>();
    full.rsplit("::").next().unwrap_or(full).to_string()
}
