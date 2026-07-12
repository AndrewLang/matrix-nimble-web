use sqlx::{Executor, Row, SqlitePool};

use crate::data::postgres::PostgresEntity;
use crate::data::provider::{DataError, DataResult};
use crate::data::schema::ColumnDef;

#[derive(Clone)]
pub struct SqliteMigrator {
    pool: SqlitePool,
}

impl SqliteMigrator {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn migrate<E: PostgresEntity>(&self) -> DataResult<()> {
        let table_name = E::plural_name();
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)")
                .bind(&table_name)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DataError::Provider(e.to_string()))?;

        log::trace!("Migrate entity {}, table exists: {}", E::plural_name(), exists);

        if !exists {
            self.create_table::<E>(&table_name).await
        } else {
            self.update_table::<E>(&table_name).await
        }
    }

    async fn create_table<E: PostgresEntity>(&self, table_name: &str) -> DataResult<()> {
        let columns = E::table_columns();
        let sql = MigrationBuilder::build_create_table(table_name, &columns);

        log::trace!("Create table SQL: {}", sql);

        self.pool.execute(sql.as_str()).await.map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(())
    }

    async fn update_table<E: PostgresEntity>(&self, table_name: &str) -> DataResult<()> {
        let defined_columns = E::table_columns();

        let rows = sqlx::query(&format!("PRAGMA table_info(\"{}\")", table_name.replace('"', "\"\"")))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        let existing_columns: Vec<String> = rows.iter().map(|row| row.get::<String, _>("name")).collect();

        for col in defined_columns {
            if !existing_columns.contains(&col.name.to_string()) {
                let sql = MigrationBuilder::build_add_column(table_name, &col);
                log::trace!("Add column SQL: {}", sql);
                self.pool.execute(sql.as_str()).await.map_err(|e| DataError::Provider(e.to_string()))?;
            }
        }

        Ok(())
    }
}

pub struct MigrationBuilder;

impl MigrationBuilder {
    pub fn build_create_table(table_name: &str, columns: &[ColumnDef]) -> String {
        let mut sql = format!("CREATE TABLE {} (", table_name);

        for (idx, col) in columns.iter().enumerate() {
            if idx > 0 {
                sql.push_str(", ");
            }
            sql.push_str(col.name);
            sql.push(' ');
            sql.push_str(&MigrationBuilder::sqlite_type(&col.data_type));

            if col.is_primary_key {
                sql.push_str(" PRIMARY KEY");
            }
            if !col.is_nullable && !col.is_primary_key {
                sql.push_str(" NOT NULL");
            }
            if col.unique {
                sql.push_str(" UNIQUE");
            }
            if let Some(default) = col.default {
                sql.push_str(" DEFAULT ");
                sql.push_str(&MigrationBuilder::sqlite_default(default));
            }
        }
        sql.push(')');
        sql
    }

    pub fn build_add_column(table_name: &str, col: &ColumnDef) -> String {
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            table_name,
            col.name,
            MigrationBuilder::sqlite_type(&col.data_type)
        );

        if !col.is_nullable {
            if let Some(default) = col.default {
                sql.push_str(" NOT NULL DEFAULT ");
                sql.push_str(&MigrationBuilder::sqlite_default(default));
            } else {
                sql.push_str(" NOT NULL");
            }
        } else if let Some(default) = col.default {
            sql.push_str(" DEFAULT ");
            sql.push_str(&MigrationBuilder::sqlite_default(default));
        }

        if col.unique {
            sql.push_str(" UNIQUE");
        }

        sql
    }

    pub fn sqlite_type(data_type: &crate::data::schema::ColumnType) -> String {
        use crate::data::schema::ColumnType;
        match data_type {
            ColumnType::Boolean | ColumnType::Integer | ColumnType::BigInt => "INTEGER".into(),
            ColumnType::Float | ColumnType::Double => "REAL".into(),
            ColumnType::Bytes => "BLOB".into(),
            ColumnType::Text
            | ColumnType::Varchar(_)
            | ColumnType::Timestamp
            | ColumnType::Uuid
            | ColumnType::Json
            | ColumnType::Custom(_) => "TEXT".into(),
        }
    }

    pub fn sqlite_default(default: &str) -> String {
        match default.trim().to_ascii_lowercase().as_str() {
        "gen_random_uuid()" => "(lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab',abs(random()) % 4 + 1,1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6))))".into(),
        "now()" => "CURRENT_TIMESTAMP".into(),
        _ => default.to_string(),
    }
    }
}
