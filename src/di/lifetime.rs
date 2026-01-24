#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    Singleton,
    Scoped,
    Transient,
}
