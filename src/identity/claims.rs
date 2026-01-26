use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Claims {
    roles: HashSet<String>,
    permissions: HashSet<String>,
    attributes: HashMap<String, String>,
}

impl Claims {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn roles(&self) -> &HashSet<String> {
        &self.roles
    }

    pub fn permissions(&self) -> &HashSet<String> {
        &self.permissions
    }

    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    pub fn add_role(mut self, role: impl Into<String>) -> Self {
        self.roles.insert(role.into());
        self
    }

    pub fn add_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.insert(permission.into());
        self
    }

    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}
