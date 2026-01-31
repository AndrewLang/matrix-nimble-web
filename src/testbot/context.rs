use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct TestContext {
    pub access_token: Option<String>,
    pub vars: HashMap<String, Value>,
    assertion_failures: Vec<String>,
}

impl TestContext {
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.vars.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.vars.get(key)
    }

    pub fn record_assertion_failure(&mut self, message: impl Into<String>) {
        self.assertion_failures.push(message.into());
    }

    pub fn take_assertion_failures(&mut self) -> Vec<String> {
        std::mem::take(&mut self.assertion_failures)
    }
}
