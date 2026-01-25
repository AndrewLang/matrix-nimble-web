use std::collections::HashMap;

use crate::entity::entity::Entity;
use crate::entity::metadata::EntityMetadata;
use crate::openapi::model::SchemaRef;
use crate::openapi::schema::{OpenApiSchema, SchemaRegistry};

#[derive(Clone, Debug)]
pub struct OpenApiRegistry {
    entries: Vec<OpenApiEntry>,
    schemas: SchemaRegistry,
    entities: HashMap<&'static str, EntityMetadata>,
}

#[derive(Clone, Debug)]
pub struct OpenApiEntry {
    pub method: String,
    pub path: String,
    pub metadata: OpenApiOperationMetadata,
}

#[derive(Clone, Debug, Default)]
pub struct OpenApiOperationMetadata {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub request_body: Option<SchemaRef>,
    pub responses: HashMap<u16, SchemaRef>,
    pub path_params: HashMap<String, SchemaRef>,
    pub query_params: HashMap<String, SchemaRef>,
    pub requires_auth: bool,
}

impl OpenApiOperationMetadata {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OpenApiRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            entries: Vec::new(),
            schemas: SchemaRegistry::new(),
            entities: HashMap::new(),
        };
        registry.schemas.register::<String>();
        registry
    }

    pub fn add(&mut self, method: &str, path: &str) {
        self.add_with_metadata(method, path, OpenApiOperationMetadata::default());
    }

    pub fn add_with_metadata(
        &mut self,
        method: &str,
        path: &str,
        metadata: OpenApiOperationMetadata,
    ) {
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|entry| entry.method == method && entry.path == path)
        {
            if metadata.operation_id.is_some() {
                existing.metadata.operation_id = metadata.operation_id;
            }
            if metadata.summary.is_some() {
                existing.metadata.summary = metadata.summary;
            }
            if metadata.description.is_some() {
                existing.metadata.description = metadata.description;
            }
            if !metadata.tags.is_empty() {
                existing.metadata.tags = metadata.tags;
            }
            if metadata.request_body.is_some() {
                existing.metadata.request_body = metadata.request_body;
            }
            if !metadata.responses.is_empty() {
                existing.metadata.responses = metadata.responses;
            }
            if !metadata.path_params.is_empty() {
                existing.metadata.path_params = metadata.path_params;
            }
            if !metadata.query_params.is_empty() {
                existing.metadata.query_params = metadata.query_params;
            }
            if metadata.requires_auth {
                existing.metadata.requires_auth = true;
            }
            return;
        }

        self.entries.push(OpenApiEntry {
            method: method.to_string(),
            path: path.to_string(),
            metadata,
        });
    }

    pub fn entries(&self) -> &[OpenApiEntry] {
        &self.entries
    }

    pub fn register_schema<T: OpenApiSchema + 'static>(&mut self) -> SchemaRef {
        self.schemas.register::<T>()
    }

    pub fn schemas(&self) -> &SchemaRegistry {
        &self.schemas
    }

    pub fn register_entity<T: Entity>(&mut self) {
        let metadata = EntityMetadata::of::<T>();
        self.schemas.set_type_name::<T>(metadata.name());
        self.entities.entry(metadata.name()).or_insert(metadata);
    }

    pub fn entities(&self) -> impl Iterator<Item = &EntityMetadata> {
        self.entities.values()
    }

    pub fn merge(&mut self, registry: OpenApiRegistry) {
        for entry in registry.entries {
            self.add_with_metadata(&entry.method, &entry.path, entry.metadata);
        }
        self.schemas.merge(&registry.schemas);
        for (name, metadata) in registry.entities {
            self.entities.entry(name).or_insert(metadata);
        }
    }
}

impl Default for OpenApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}
