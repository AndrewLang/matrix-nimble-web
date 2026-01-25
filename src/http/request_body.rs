#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody {
    Empty,
    Bytes(Vec<u8>),
    Text(String),
}

impl Default for RequestBody {
    fn default() -> Self {
        RequestBody::Empty
    }
}
