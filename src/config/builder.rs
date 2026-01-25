use std::collections::HashMap;

use crate::config::config::Configuration;
use crate::config::env::EnvConfigSource;
use crate::config::file::FileSource;
use crate::config::source::ConfigSource;

pub struct ConfigBuilder {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    pub fn with_json<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.sources
            .push(Box::new(FileSource::json(path.as_ref().to_path_buf())));
        self
    }

    pub fn with_toml<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.sources
            .push(Box::new(FileSource::toml(path.as_ref().to_path_buf())));
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
                values.insert(key, value);
            }
        }
        Configuration::new(values)
    }
}
