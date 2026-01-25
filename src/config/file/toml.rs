use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use toml::Value;

use crate::config::source::ConfigSource;

pub struct TomlFileSource {
    path: PathBuf,
}

impl TomlFileSource {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn flatten_toml(&self, prefix: Option<&str>, value: &Value, out: &mut HashMap<String, String>) {
        match value {
            Value::Table(map) => {
                for (key, child) in map {
                    let next = match prefix {
                        Some(p) => format!("{}.{}", p, key),
                        None => key.to_string(),
                    };
                    self.flatten_toml(Some(&next), child, out);
                }
            }
            Value::Array(items) => {
                for (idx, child) in items.iter().enumerate() {
                    let next = match prefix {
                        Some(p) => format!("{}.{}", p, idx),
                        None => idx.to_string(),
                    };
                    self.flatten_toml(Some(&next), child, out);
                }
            }
            other => {
                if let Some(key) = prefix {
                    let text = match other {
                        Value::String(s) => s.clone(),
                        Value::Boolean(b) => b.to_string(),
                        Value::Integer(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Datetime(dt) => dt.to_string(),
                        _ => other.to_string(),
                    };
                    out.insert(key.to_string(), text);
                }
            }
        }
    }
}

impl ConfigSource for TomlFileSource {
    fn load(&self) -> HashMap<String, String> {
        let content = fs::read_to_string(&self.path).expect("toml file read failed");
        let content = content.trim_start_matches('\u{feff}');
        let value: Value = content.parse::<Value>().expect("toml parse failed");
        let mut out = HashMap::new();
        self.flatten_toml(None, &value, &mut out);
        out
    }
}
