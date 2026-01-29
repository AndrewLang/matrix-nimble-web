#![cfg(feature = "postgres")]

use nimble_web::app::builder::AppBuilder;
use std::env;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock")
}

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

#[tokio::test]
#[cfg(feature = "postgres")]
async fn app_builder_registers_postgres_components() {
    let _lock = env_lock();
    // Use a valid connection string structure, even if the DB doesn't exist
    let _guard = EnvGuard::set(
        "NIMBLE_POSTGRES_URL",
        "postgres://user:pass@localhost:5432/testdb",
    );

    let mut builder = AppBuilder::new();
    builder.use_env().use_postgres();
    let app = builder.build();

    let pool = app.services().resolve::<sqlx::PgPool>();
    assert!(pool.is_some(), "PgPool should be registered");

    let migrator = app
        .services()
        .resolve::<nimble_web::data::postgres::migration::PostgresMigrator>();
    assert!(migrator.is_some(), "PostgresMigrator should be registered");

    // Check config works
    let config = app
        .services()
        .resolve::<nimble_web::Configuration>()
        .expect("config");
    let pg_config = config.postgres_config();
    assert_eq!(pg_config.url, "postgres://user:pass@localhost:5432/testdb");
}
