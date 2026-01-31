#[derive(Debug)]
pub struct TestError {
    pub message: String,
}

impl TestError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self { message: s.into() }
    }
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TestError {}
