use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::config::source::ConfigSource;

pub struct JsonFileSource {
    path: PathBuf,
}

impl JsonFileSource {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn flatten_json(&self, prefix: Option<&str>, value: &Value, out: &mut HashMap<String, String>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let next = match prefix {
                        Some(p) => format!("{}.{}", p, key),
                        None => key.to_string(),
                    };
                    self.flatten_json(Some(&next), child, out);
                }
            }
            Value::Array(items) => {
                for (idx, child) in items.iter().enumerate() {
                    let next = match prefix {
                        Some(p) => format!("{}.{}", p, idx),
                        None => idx.to_string(),
                    };
                    self.flatten_json(Some(&next), child, out);
                }
            }
            other => {
                if let Some(key) = prefix {
                    let text = match other {
                        Value::String(s) => s.clone(),
                        Value::Bool(b) => b.to_string(),
                        Value::Number(n) => n.to_string(),
                        Value::Null => "null".to_string(),
                        _ => other.to_string(),
                    };
                    out.insert(key.to_string(), text);
                }
            }
        }
    }
}

impl ConfigSource for JsonFileSource {
    fn load(&self) -> HashMap<String, String> {
        let content = fs::read_to_string(&self.path).expect("json file read failed");
        let content = content.trim_start_matches('\u{feff}');
        let value: Value = serde_json::from_str(content).expect("json parse failed");
        let mut out = HashMap::new();
        self.flatten_json(None, &value, &mut out);
        out
    }
}
