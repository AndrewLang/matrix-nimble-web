use crate::validation::ValidationError;

pub trait Validator<T>: Send + Sync {
    fn validate(&self, value: &T) -> Result<(), ValidationError>;
}
