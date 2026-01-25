use std::collections::HashMap;

use crate::openapi::model::{
    Components, MediaType, OpenApiDocument, OpenApiInfo, Operation, Parameter, PathItem,
    RequestBody, Response, SecurityScheme,
};
use crate::openapi::registry::OpenApiRegistry;

pub struct OpenApiGenerator;

impl OpenApiGenerator {
    pub fn generate(registry: &OpenApiRegistry) -> OpenApiDocument {
        let mut paths: HashMap<String, PathItem> = HashMap::new();

        let mut requires_auth = false;

        for entry in registry.entries() {
            let method = entry.method.to_lowercase();
            let path_item = paths
                .entry(entry.path.clone())
                .or_insert_with(|| PathItem {
                    operations: HashMap::new(),
                });
            let parameters = build_parameters(entry);
            let security = if entry.metadata.requires_auth {
                requires_auth = true;
                let mut requirement = HashMap::new();
                requirement.insert("bearerAuth".to_string(), Vec::new());
                vec![requirement]
            } else {
                Vec::new()
            };
            path_item.operations.entry(method).or_insert_with(|| Operation {
                operation_id: entry.metadata.operation_id.clone(),
                summary: entry.metadata.summary.clone(),
                description: entry.metadata.description.clone(),
                tags: entry.metadata.tags.clone(),
                request_body: entry.metadata.request_body.as_ref().map(|schema_ref| {
                    let mut content = HashMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: Some(schema_ref.clone()),
                        },
                    );
                    RequestBody { content }
                }),
                responses: entry
                    .metadata
                    .responses
                    .iter()
                    .map(|(status, schema_ref)| {
                        let mut content = HashMap::new();
                        content.insert(
                            "application/json".to_string(),
                            MediaType {
                                schema: Some(schema_ref.clone()),
                            },
                        );
                        (
                            status.to_string(),
                            Response {
                                description: "response".to_string(),
                                content,
                            },
                        )
                    })
                    .collect(),
                parameters,
                security,
            });
        }

        // Ensure the OpenAPI endpoint is listed.
        let path_item = paths
            .entry("/openapi.json".to_string())
            .or_insert_with(|| PathItem {
                operations: HashMap::new(),
            });
        path_item
            .operations
            .entry("get".to_string())
            .or_insert_with(Operation::default);

        let mut security_schemes = HashMap::new();
        if requires_auth {
            security_schemes.insert(
                "bearerAuth".to_string(),
                SecurityScheme {
                    kind: "http".to_string(),
                    scheme: Some("bearer".to_string()),
                },
            );
        }

        OpenApiDocument {
            openapi: "3.1.0".to_string(),
            info: OpenApiInfo {
                title: "nimble-web".to_string(),
                version: "0.1.0".to_string(),
            },
            paths,
            components: Components {
                schemas: registry.schemas().schemas().clone(),
                security_schemes,
            },
        }
    }
}

fn build_parameters(entry: &crate::openapi::registry::OpenApiEntry) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for name in extract_path_params(&entry.path) {
        let schema = entry
            .metadata
            .path_params
            .get(&name)
            .cloned()
            .unwrap_or_else(default_string_schema_ref);
        if seen.insert((name.clone(), "path")) {
            parameters.push(Parameter {
                name,
                location: "path".to_string(),
                required: true,
                schema,
            });
        }
    }

    for (name, schema) in &entry.metadata.query_params {
        if seen.insert((name.clone(), "query")) {
            parameters.push(Parameter {
                name: name.clone(),
                location: "query".to_string(),
                required: false,
                schema: schema.clone(),
            });
        }
    }

    parameters
}

fn extract_path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') && segment.len() > 2 {
                Some(segment[1..segment.len() - 1].to_string())
            } else {
                None
            }
        })
        .collect()
}

fn default_string_schema_ref() -> crate::openapi::model::SchemaRef {
    crate::openapi::model::SchemaRef {
        ref_path: "#/components/schemas/String".to_string(),
    }
}
