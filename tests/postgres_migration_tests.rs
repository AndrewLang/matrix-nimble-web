#![cfg(feature = "postgres")]

use nimble_web::data::postgres::migration::MigrationBuilder;
use nimble_web::data::schema::{ColumnDef, ColumnType};

#[test]
fn test_build_create_table() {
    let columns = vec![
        ColumnDef::new("id", ColumnType::Uuid).primary_key(),
        ColumnDef::new("name", ColumnType::Text).not_null(),
        ColumnDef::new("email", ColumnType::Text).unique(),
    ];
    let sql = MigrationBuilder::build_create_table("users", &columns);
    assert_eq!(
        sql,
        "CREATE TABLE users (id UUID PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE)"
    );
}

#[test]
fn test_build_add_column() {
    let col = ColumnDef::new("age", ColumnType::Integer).default("0");
    let sql = MigrationBuilder::build_add_column("users", &col);
    assert_eq!(sql, "ALTER TABLE users ADD COLUMN age INTEGER DEFAULT 0");
}

#[test]
fn test_build_add_column_not_null_default() {
    let col = ColumnDef::new("active", ColumnType::Boolean)
        .not_null()
        .default("true");
    let sql = MigrationBuilder::build_add_column("users", &col);
    assert_eq!(
        sql,
        "ALTER TABLE users ADD COLUMN active BOOLEAN NOT NULL DEFAULT true"
    );
}

#[test]
fn test_build_add_column_unique() {
    let col = ColumnDef::new("code", ColumnType::Text).unique();
    let sql = MigrationBuilder::build_add_column("products", &col);
    assert_eq!(sql, "ALTER TABLE products ADD COLUMN code TEXT UNIQUE");
}
