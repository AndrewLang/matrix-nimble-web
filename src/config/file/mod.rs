use std::path::PathBuf;

use crate::config::file::json::JsonFileSource;
use crate::config::file::toml::TomlFileSource;
use crate::config::source::ConfigSource;

pub mod json;
pub mod toml;

pub enum FileSource {
    Json(PathBuf),
    Toml(PathBuf),
}

impl FileSource {
    pub fn json(path: PathBuf) -> impl ConfigSource {
        JsonFileSource::new(path)
    }

    pub fn toml(path: PathBuf) -> impl ConfigSource {
        TomlFileSource::new(path)
    }
}
