use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub(crate) fn new(values: HashMap<String, String>) -> Self {
        Self { values }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|value| value.as_str())
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key)?.parse().ok()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
}
