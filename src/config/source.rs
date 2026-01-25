use std::collections::HashMap;

pub trait ConfigSource: Send + Sync {
    fn load(&self) -> HashMap<String, String>;
}
