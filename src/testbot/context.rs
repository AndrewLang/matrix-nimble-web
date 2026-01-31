use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct TestContext {
    pub access_token: Option<String>,
    pub vars: HashMap<String, Value>,
}

impl TestContext {
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.vars.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.vars.get(key)
    }
}
