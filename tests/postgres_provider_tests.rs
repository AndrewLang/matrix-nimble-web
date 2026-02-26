#![cfg(feature = "postgres")]

use nimble_web::data::paging::PageRequest;
use nimble_web::data::postgres::{PostgresEntity, PostgresProvider};
use nimble_web::data::provider::{DataError, DataProvider};
use nimble_web::data::query::{
    Filter, FilterOperator, GroupBy, Join, JoinOn, Query, Sort, SortDirection, Value,
};
use nimble_web::data::schema::ColumnDef;
use nimble_web::entity::entity::Entity;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::FromRow;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Photo {
    id: i64,
}

impl Entity for Photo {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "photo"
    }

    fn plural_name() -> String {
        "photos".to_string()
    }
}

#[derive(Debug, Clone)]
struct PgItem {
    id: i64,
    name: String,
}

impl Entity for PgItem {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "pg_item"
    }

    fn plural_name() -> String {
        "pg_items".to_string()
    }
}

impl PostgresEntity for PgItem {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Int(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "name"]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![Value::Int(self.id), Value::String(self.name.clone())]
    }

    fn update_columns() -> &'static [&'static str] {
        &["name"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![Value::String(self.name.clone())]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![]
    }
}

impl<'r> FromRow<'r, PgRow> for PgItem {
    fn from_row(_row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }
}

#[derive(Debug, Clone)]
struct BadPgItem {
    id: i64,
}

impl Entity for BadPgItem {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "bad_item"
    }
}

impl PostgresEntity for BadPgItem {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Int(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "name"]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![Value::Int(self.id)]
    }

    fn update_columns() -> &'static [&'static str] {
        &["name", "status"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![Value::String("only-name".to_string())]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![]
    }
}

impl<'r> FromRow<'r, PgRow> for BadPgItem {
    fn from_row(_row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }
}

#[derive(Debug, Clone)]
struct PgVariety {
    id: i64,
}

impl Entity for PgVariety {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "pg_variety"
    }

    fn plural_name() -> String {
        "pg_varieties".to_string()
    }
}

impl PostgresEntity for PgVariety {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Int(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "flag", "count", "ratio", "blob", "nothing", "tags"]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Int(self.id),
            Value::Bool(true),
            Value::UInt(9),
            Value::Float(1.25),
            Value::Bytes(vec![1, 2, 3]),
            Value::Null,
            Value::List(vec![Value::String("a".to_string())]),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["flag"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![Value::Bool(false)]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![]
    }
}

impl<'r> FromRow<'r, PgRow> for PgVariety {
    fn from_row(_row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }
}

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(200))
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/postgres")
        .expect("lazy pool")
}

#[test]
fn select_uses_entity_plural_name() {
    let query = Query::<Photo>::new();
    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("FROM photos t"));
}

#[test]
fn filters_translate_to_where_clause() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "status".to_string(),
        operator: FilterOperator::Eq,
        value: Value::String("active".to_string()),
    });
    query.filters.push(Filter {
        field: "size".to_string(),
        operator: FilterOperator::Gt,
        value: Value::Int(10),
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("WHERE status = ? AND size > ?"));
}

#[test]
fn sorting_translates_to_order_by() {
    let mut query = Query::<Photo>::new();
    query.sorting.push(Sort {
        field: "created_at".to_string(),
        direction: SortDirection::Desc,
    });
    query.sorting.push(Sort {
        field: "id".to_string(),
        direction: SortDirection::Asc,
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("ORDER BY created_at DESC, id ASC"));
}

#[test]
fn paging_translates_to_limit_offset() {
    let mut query = Query::<Photo>::new();
    query.paging = Some(PageRequest::new(2, 10));

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("LIMIT 10 OFFSET 10"));
}

#[test]
fn group_by_translates_to_group_by_clause() {
    let mut query = Query::<Photo>::new();
    query.group_by = Some(GroupBy {
        fields: vec!["category".to_string()],
        aggregates: Vec::new(),
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("GROUP BY category"));
}

#[test]
fn count_sql_includes_group_by_when_present() {
    let mut query = Query::<Photo>::new();
    query.group_by = Some(GroupBy {
        fields: vec!["category".to_string()],
        aggregates: Vec::new(),
    });

    let sql = PostgresProvider::<Photo>::build_count_sql(&query);
    assert!(sql.contains("SELECT COUNT(*) FROM (SELECT 1 FROM photos t"));
    assert!(sql.contains("GROUP BY category"));
    assert!(sql.contains(") AS grouped"));
}

#[test]
fn in_and_between_filters_translate() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::In,
        value: Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
    });
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::NotIn,
        value: Value::List(vec![Value::Int(4)]),
    });
    query.filters.push(Filter {
        field: "created_at".to_string(),
        operator: FilterOperator::Between,
        value: Value::List(vec![Value::Int(10), Value::Int(20)]),
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("id IN (?, ?, ?)"));
    assert!(sql.contains("id NOT IN (?)"));
    assert!(sql.contains("created_at BETWEEN ? AND ?"));
}

#[test]
fn like_and_null_filters_translate() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "name".to_string(),
        operator: FilterOperator::Contains,
        value: Value::String("sun".to_string()),
    });
    query.filters.push(Filter {
        field: "deleted_at".to_string(),
        operator: FilterOperator::IsNull,
        value: Value::Null,
    });
    query.filters.push(Filter {
        field: "archived_at".to_string(),
        operator: FilterOperator::IsNotNull,
        value: Value::Null,
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("name LIKE ?"));
    assert!(sql.contains("deleted_at IS NULL"));
    assert!(sql.contains("archived_at IS NOT NULL"));
}

#[test]
fn eq_and_ne_null_translate_to_is_null() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "deleted_at".to_string(),
        operator: FilterOperator::Eq,
        value: Value::Null,
    });
    query.filters.push(Filter {
        field: "archived_at".to_string(),
        operator: FilterOperator::Ne,
        value: Value::Null,
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("deleted_at IS NULL"));
    assert!(sql.contains("archived_at IS NOT NULL"));
}

#[test]
fn starts_and_ends_with_translate_to_like() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "title".to_string(),
        operator: FilterOperator::StartsWith,
        value: Value::String("pre".to_string()),
    });
    query.filters.push(Filter {
        field: "title".to_string(),
        operator: FilterOperator::EndsWith,
        value: Value::String("suf".to_string()),
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("title LIKE ?"));
}

#[test]
fn empty_in_list_generates_placeholder() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::In,
        value: Value::List(Vec::new()),
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("id IN (?)"));
}

#[test]
fn empty_group_by_is_ignored() {
    let mut query = Query::<Photo>::new();
    query.group_by = Some(GroupBy {
        fields: Vec::new(),
        aggregates: Vec::new(),
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(!sql.contains("GROUP BY"));
}

#[test]
fn paging_with_zero_values_uses_defaults() {
    let mut query = Query::<Photo>::new();
    query.paging = Some(PageRequest::new(0, 0));

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("LIMIT 1 OFFSET 0"));
}

#[test]
fn join_translates_to_join_clause() {
    let mut query = Query::<Photo>::new();
    query.joins.push(Join {
        entity_name: "albums".to_string(),
        alias: Some("a".to_string()),
        on: vec![JoinOn {
            left: "t.album_id".to_string(),
            operator: FilterOperator::Eq,
            right: "a.id".to_string(),
        }],
    });

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("JOIN albums a ON t.album_id = a.id"));
}

#[test]
fn distinct_select_expression_translates() {
    let query = nimble_web::data::query_builder::QueryBuilder::<Photo>::new()
        .distinct()
        .select_as("to_char(p.day_date, 'YYYY')", "year")
        .sort_desc("year")
        .build();

    let sql = PostgresProvider::<Photo>::build_select_sql(&query);
    assert!(sql.contains("SELECT DISTINCT to_char(p.day_date, 'YYYY') AS year"));
    assert!(sql.contains("FROM photos t"));
    assert!(sql.contains("ORDER BY year DESC"));
}

#[tokio::test]
async fn create_update_reject_mismatched_columns() {
    let pool = lazy_pool();
    let provider = PostgresProvider::<BadPgItem>::new(pool);

    let create = provider.create(BadPgItem { id: 1 }).await;
    assert!(matches!(create, Err(DataError::InvalidQuery(_))));

    let update = provider.update(BadPgItem { id: 1 }).await;
    assert!(matches!(update, Err(DataError::InvalidQuery(_))));
}

#[tokio::test]
async fn query_rejects_invalid_filter_shapes() {
    let pool = lazy_pool();
    let provider = PostgresProvider::<PgItem>::new(pool);

    let mut query = Query::<PgItem>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::In,
        value: Value::String("bad".to_string()),
    });
    let result = provider.query(query).await;
    assert!(matches!(result, Err(DataError::InvalidQuery(_))));

    let mut query = Query::<PgItem>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::Between,
        value: Value::List(vec![Value::Int(1)]),
    });
    let result = provider.query(query).await;
    assert!(matches!(result, Err(DataError::InvalidQuery(_))));
}

#[tokio::test]
async fn provider_methods_surface_connection_errors() {
    let pool = lazy_pool();
    let provider = PostgresProvider::<PgItem>::new(pool);
    let entity = PgItem {
        id: 1,
        name: "one".to_string(),
    };

    let create = tokio::time::timeout(Duration::from_secs(1), provider.create(entity.clone()))
        .await
        .expect("timeout");
    assert!(matches!(create, Err(DataError::Provider(_))));

    let get = tokio::time::timeout(Duration::from_secs(1), provider.get(&1))
        .await
        .expect("timeout");
    assert!(matches!(get, Err(DataError::Provider(_))));

    let update = tokio::time::timeout(Duration::from_secs(1), provider.update(entity.clone()))
        .await
        .expect("timeout");
    assert!(matches!(update, Err(DataError::Provider(_))));

    let delete = tokio::time::timeout(Duration::from_secs(1), provider.delete(&1))
        .await
        .expect("timeout");
    assert!(matches!(delete, Err(DataError::Provider(_))));

    let query = tokio::time::timeout(Duration::from_secs(1), provider.query(Query::new()))
        .await
        .expect("timeout");
    assert!(matches!(query, Err(DataError::Provider(_))));
}

#[tokio::test]
async fn create_binds_all_value_types() {
    let pool = lazy_pool();
    let provider = PostgresProvider::<PgVariety>::new(pool);
    let entity = PgVariety { id: 1 };

    let result = tokio::time::timeout(Duration::from_secs(1), provider.create(entity))
        .await
        .expect("timeout");
    assert!(matches!(result, Err(DataError::Provider(_))));
}

#[tokio::test]
async fn query_with_join_sort_and_paging_hits_builder_paths() {
    let pool = lazy_pool();
    let provider = PostgresProvider::<PgItem>::with_schema(pool, "public");

    let mut query = Query::<PgItem>::new();
    query.joins.push(Join {
        entity_name: "owners".to_string(),
        alias: Some("o".to_string()),
        on: vec![JoinOn {
            left: "t.owner_id".to_string(),
            operator: FilterOperator::Eq,
            right: "o.id".to_string(),
        }],
    });
    query.filters.push(Filter {
        field: "t.name".to_string(),
        operator: FilterOperator::Eq,
        value: Value::Null,
    });
    query.sorting.push(Sort {
        field: "t.name".to_string(),
        direction: SortDirection::Asc,
    });
    query.paging = Some(PageRequest::new(1, 5));

    let result = tokio::time::timeout(Duration::from_secs(1), provider.query(query))
        .await
        .expect("timeout");
    assert!(matches!(result, Err(DataError::Provider(_))));
}
