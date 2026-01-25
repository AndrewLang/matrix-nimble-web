use std::sync::Arc;

use crate::security::policy::Policy;
use crate::validation::AnyValidator;

#[derive(Clone)]
pub struct EndpointMetadata {
    method: String,
    route_pattern: String,
    name: Option<String>,
    tags: Vec<String>,
    policy: Option<Policy>,
    validators: Vec<Arc<dyn AnyValidator>>,
}

impl EndpointMetadata {
    pub fn new(method: &str, route_pattern: &str) -> Self {
        Self {
            method: method.to_string(),
            route_pattern: route_pattern.to_string(),
            name: None,
            tags: Vec::new(),
            policy: None,
            validators: Vec::new(),
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn route_pattern(&self) -> &str {
        &self.route_pattern
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn require_policy(mut self, policy: Policy) -> Self {
        self.policy = Some(policy);
        self
    }

    pub fn policy(&self) -> Option<&Policy> {
        self.policy.as_ref()
    }

    pub fn add_validator<T>(mut self, validator: T) -> Self
    where
        T: AnyValidator + 'static,
    {
        self.validators.push(Arc::new(validator));
        self
    }

    pub fn validators(&self) -> &[Arc<dyn AnyValidator>] {
        &self.validators
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }
}
