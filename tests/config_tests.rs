use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use nimble_web::config::ConfigBuilder;

#[derive(Debug)]
struct EnvGuard {
    key: String,
    prev: Option<String>,
}

impl EnvGuard {
    fn set(key: &str, value: &str) -> Self {
        let prev = env::var(key).ok();
        env::set_var(key, value);
        Self {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.prev {
            env::set_var(&self.key, value);
        } else {
            env::remove_var(&self.key);
        }
    }
}

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().expect("env lock")
}

#[test]
fn layering_overrides_previous_sources() {
    let config = ConfigBuilder::new()
        .with_json(data_path("config.json"))
        .with_toml(data_path("config.toml"))
        .build();

    assert_eq!(config.get("server.port"), Some("8080"));
}

#[test]
fn json_flattens_to_dot_keys() {
    let config = ConfigBuilder::new()
        .with_json(data_path("config.json"))
        .build();

    assert_eq!(
        config.get("ConnectionStrings.DefaultConnection"),
        Some("Host=localhost;Port=5432;Database=mrp;Username=mrp_user;Password=mrp_dev_password")
    );
    assert_eq!(config.get("Jwt.Key"), Some("jwt-key"));
    assert_eq!(config.get("Jwt.Issuer"), Some("mrp-issuer"));
    assert_eq!(config.get("Jwt.Audience"), Some("mrp-audience"));
    assert_eq!(config.get("Jwt.AccessTokenMinutes"), Some("60"));
    assert_eq!(config.get("server.host"), Some("127.0.0.1"));
    assert_eq!(config.get("server.port"), Some("3000"));
    assert_eq!(config.get("upload.max.mb"), Some("50"));
    assert_eq!(config.get("flags.enabled"), Some("true"));
}

#[test]
fn toml_flattens_to_dot_keys() {
    let config = ConfigBuilder::new()
        .with_toml(data_path("config.toml"))
        .build();

    assert_eq!(config.get("server.host"), Some("127.0.0.1"));
    assert_eq!(config.get("server.port"), Some("8080"));
    assert_eq!(config.get("upload.max.mb"), Some("75"));
    assert_eq!(config.get("flags.enabled"), Some("false"));
}

#[test]
fn env_overrides_file_values_with_prefix() {
    let _lock = env_lock();
    let _guard = EnvGuard::set("NIMBLE_UPLOAD_MAX_MB", "100");

    let config = ConfigBuilder::new()
        .with_json(data_path("config.json"))
        .with_env("NIMBLE_")
        .build();

    assert_eq!(config.get("upload.max.mb"), Some("100"));
}

#[test]
fn env_prefix_ignores_unrelated_keys() {
    let _lock = env_lock();
    let _guard = EnvGuard::set("OTHER_UPLOAD_MAX_MB", "200");

    let config = ConfigBuilder::new()
        .with_json(data_path("config.json"))
        .with_env("NIMBLE_")
        .build();

    assert_eq!(config.get("upload.max.mb"), Some("50"));
}

#[test]
fn env_override_wins_over_toml() {
    let _lock = env_lock();
    let _guard = EnvGuard::set("NIMBLE_SERVER_PORT", "9000");

    let config = ConfigBuilder::new()
        .with_json(data_path("config.json"))
        .with_toml(data_path("config.toml"))
        .with_env("NIMBLE_")
        .build();

    assert_eq!(config.get("server.port"), Some("9000"));
}

#[test]
fn typed_access_handles_false_and_invalid_numbers() {
    let config = ConfigBuilder::new()
        .with_toml(data_path("config.toml"))
        .build();

    assert_eq!(config.get_bool("flags.enabled"), Some(false));
    assert_eq!(config.get_u64("server.host"), None);
}

#[test]
fn typed_access_and_missing_keys() {
    let config = ConfigBuilder::new()
        .with_json(data_path("config.json"))
        .build();

    assert_eq!(config.get_u64("server.port"), Some(3000));
    assert_eq!(config.get_bool("flags.enabled"), Some(true));
    assert_eq!(config.get("missing.key"), None);
    assert_eq!(config.get_u64("missing.key"), None);
    assert_eq!(config.get_bool("missing.key"), None);
}
