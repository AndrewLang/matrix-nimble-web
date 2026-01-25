use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenApiDocument {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: HashMap<String, PathItem>,
    pub components: Components,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(flatten)]
    pub operations: HashMap<String, Operation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Operation {
    #[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub security: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Components {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub schemas: HashMap<String, Schema>,
    #[serde(
        rename = "securitySchemes",
        skip_serializing_if = "HashMap::is_empty",
        default
    )]
    pub security_schemes: HashMap<String, SecurityScheme>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RequestBody {
    pub content: HashMap<String, MediaType>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub content: HashMap<String, MediaType>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaRef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub required: bool,
    pub schema: SchemaRef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub properties: HashMap<String, Schema>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl Schema {
    pub fn object(properties: HashMap<String, Schema>, required: Vec<String>) -> Self {
        Self {
            schema_type: Some("object".to_string()),
            properties,
            required,
            items: None,
            format: None,
        }
    }

    pub fn string() -> Self {
        Self {
            schema_type: Some("string".to_string()),
            properties: HashMap::new(),
            required: Vec::new(),
            items: None,
            format: None,
        }
    }

    pub fn integer() -> Self {
        Self {
            schema_type: Some("integer".to_string()),
            properties: HashMap::new(),
            required: Vec::new(),
            items: None,
            format: None,
        }
    }

    pub fn integer_with_format(format: &str) -> Self {
        Self {
            schema_type: Some("integer".to_string()),
            properties: HashMap::new(),
            required: Vec::new(),
            items: None,
            format: Some(format.to_string()),
        }
    }
}
