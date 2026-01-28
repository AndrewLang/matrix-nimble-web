#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Boolean,
    Integer,
    BigInt,
    Float,
    Double,
    Text,
    Varchar(u32),
    Bytes,
    Timestamp,
    Uuid,
    Json,
    Custom(&'static str),
}

impl ColumnType {
    pub fn to_sql(&self) -> String {
        match self {
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::Integer => "INTEGER".to_string(),
            ColumnType::BigInt => "BIGINT".to_string(),
            ColumnType::Float => "REAL".to_string(),
            ColumnType::Double => "DOUBLE PRECISION".to_string(),
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Varchar(len) => format!("VARCHAR({})", len),
            ColumnType::Bytes => "BYTEA".to_string(),
            ColumnType::Timestamp => "TIMESTAMP WITH TIME ZONE".to_string(),
            ColumnType::Uuid => "UUID".to_string(),
            ColumnType::Json => "JSONB".to_string(),
            ColumnType::Custom(t) => t.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: &'static str,
    pub data_type: ColumnType,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub unique: bool,
    pub default: Option<&'static str>,
}

impl ColumnDef {
    pub fn new(name: &'static str, data_type: ColumnType) -> Self {
        Self {
            name,
            data_type,
            is_primary_key: false,
            is_nullable: true,
            unique: false,
            default: None,
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_nullable = false;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.is_nullable = false;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn default(mut self, value: &'static str) -> Self {
        self.default = Some(value);
        self
    }
}
