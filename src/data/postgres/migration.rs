use sqlx::{Executor, PgPool, Row};

use crate::data::postgres::PostgresEntity;
use crate::data::provider::{DataError, DataResult};
use crate::data::schema::ColumnDef;

#[derive(Clone)]
pub struct PostgresMigrator {
    pool: PgPool,
}

impl PostgresMigrator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate<E: PostgresEntity>(&self) -> DataResult<()> {
        let table_name = E::plural_name();
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
        )
        .bind(&table_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DataError::Provider(e.to_string()))?;

        log::trace!(
            "Migrate entity {}, table exists: {}",
            E::plural_name(),
            exists
        );

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

        self.pool
            .execute(sql.as_str())
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(())
    }

    async fn update_table<E: PostgresEntity>(&self, table_name: &str) -> DataResult<()> {
        let defined_columns = E::table_columns();

        let rows = sqlx::query(
            "SELECT column_name FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1"
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataError::Provider(e.to_string()))?;

        log::trace!("Update table columns: {:#?}", rows);

        let existing_columns: Vec<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("column_name"))
            .collect();

        for col in defined_columns {
            if !existing_columns.contains(&col.name.to_string()) {
                let sql = MigrationBuilder::build_add_column(table_name, &col);
                log::trace!("Add column SQL: {}", sql);
                self.pool
                    .execute(sql.as_str())
                    .await
                    .map_err(|e| DataError::Provider(e.to_string()))?;
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
            sql.push_str(&col.data_type.to_sql());

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
                sql.push_str(default);
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
            col.data_type.to_sql()
        );

        if !col.is_nullable {
            if let Some(default) = col.default {
                sql.push_str(" NOT NULL DEFAULT ");
                sql.push_str(default);
            } else {
                sql.push_str(" NOT NULL");
            }
        } else if let Some(default) = col.default {
            sql.push_str(" DEFAULT ");
            sql.push_str(default);
        }

        if col.unique {
            sql.push_str(" UNIQUE");
        }

        sql
    }
}
