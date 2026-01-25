use std::collections::HashMap;
use std::env;

use crate::config::source::ConfigSource;

pub struct EnvConfigSource {
    prefix: String,
}

impl EnvConfigSource {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    fn map_key(&self, key: &str) -> String {
        let raw = key.strip_prefix(&self.prefix).unwrap_or("");
        raw.to_ascii_lowercase().replace('_', ".")
    }
}

impl ConfigSource for EnvConfigSource {
    fn load(&self) -> HashMap<String, String> {
        let mut values = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with(&self.prefix) {
                let mapped = self.map_key(&key);
                if !mapped.is_empty() {
                    values.insert(mapped, value);
                }
            }
        }
        values
    }
}
