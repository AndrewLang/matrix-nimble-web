use std::collections::HashMap;
use std::path::Path;

use crate::config::config::Configuration;
use crate::config::env::EnvConfigSource;
use crate::config::file::FileSource;
use crate::config::source::ConfigSource;

pub struct ConfigBuilder {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn with_json<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.sources
            .push(Box::new(FileSource::json(path.as_ref().to_path_buf())));
        self
    }

    pub fn with_toml<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.sources
            .push(Box::new(FileSource::toml(path.as_ref().to_path_buf())));
        self
    }

    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => {
                self.sources.push(Box::new(FileSource::toml(path)));
            }
            _ => {
                self.sources.push(Box::new(FileSource::json(path)));
            }
        }
        self
    }

    pub fn with_env(mut self, prefix: &str) -> Self {
        self.sources.push(Box::new(EnvConfigSource::new(prefix)));
        self
    }

    pub fn build(self) -> Configuration {
        let mut values = HashMap::new();
        for source in self.sources {
            let next = source.load();
            for (key, value) in next {
                values.insert(key.to_lowercase(), value);
            }
        }
        Configuration::new(values)
    }
}
