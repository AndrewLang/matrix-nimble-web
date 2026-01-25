#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointMetadata {
    method: String,
    route_pattern: String,
    name: Option<String>,
    tags: Vec<String>,
    policy: Option<crate::security::policy::Policy>,
}

impl EndpointMetadata {
    pub fn new(method: &str, route_pattern: &str) -> Self {
        Self {
            method: method.to_string(),
            route_pattern: route_pattern.to_string(),
            name: None,
            tags: Vec::new(),
            policy: None,
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

    pub fn require_policy(mut self, policy: crate::security::policy::Policy) -> Self {
        self.policy = Some(policy);
        self
    }

    pub fn policy(&self) -> Option<&crate::security::policy::Policy> {
        self.policy.as_ref()
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
