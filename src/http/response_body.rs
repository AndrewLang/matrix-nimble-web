#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseBody {
    Empty,
    Bytes(Vec<u8>),
    Text(String),
}

impl Default for ResponseBody {
    fn default() -> Self {
        ResponseBody::Empty
    }
}
