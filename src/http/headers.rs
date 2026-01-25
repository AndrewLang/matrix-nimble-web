use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct HttpHeaders {
    values: HashMap<String, String>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|value| value.as_str())
    }
}
