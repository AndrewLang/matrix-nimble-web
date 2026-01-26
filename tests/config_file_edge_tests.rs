use std::env;
use std::path::PathBuf;

use nimble_web::config::ConfigBuilder;

fn temp_path(name: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(name);
    path
}

#[test]
fn json_file_flattens_arrays() {
    let path = temp_path("nimble-json-array.json");
    let content = r#"{"items":[{"name":"a"},{"name":"b"}]}"#;
    std::fs::write(&path, content).expect("write json");

    let config = ConfigBuilder::new().with_json(&path).build();
    assert_eq!(config.get("items.0.name"), Some("a"));
    assert_eq!(config.get("items.1.name"), Some("b"));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn toml_file_flattens_arrays() {
    let path = temp_path("nimble-toml-array.toml");
    let content = r#"
items = [
  { name = "a" },
  { name = "b" }
]
"#;
    std::fs::write(&path, content).expect("write toml");

    let config = ConfigBuilder::new().with_toml(&path).build();
    assert_eq!(config.get("items.0.name"), Some("a"));
    assert_eq!(config.get("items.1.name"), Some("b"));

    let _ = std::fs::remove_file(&path);
}
