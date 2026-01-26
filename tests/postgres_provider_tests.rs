use nimble_web::data::paging::PageRequest;
use nimble_web::data::postgres::PostgresProvider;
use nimble_web::data::query::{Filter, FilterOperator, GroupBy, Query, Sort, SortDirection, Value};
use nimble_web::entity::entity::Entity;

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
