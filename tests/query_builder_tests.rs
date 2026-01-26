use nimble_web::data::paging::PageRequest;
use nimble_web::data::query::{AggregateFunction, FilterOperator, SortDirection, Value};
use nimble_web::data::query_builder::QueryBuilder;
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

#[derive(Debug, Clone)]
struct Album {
    id: i64,
}

impl Entity for Album {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "album"
    }

    fn plural_name() -> String {
        "albums".to_string()
    }
}

#[test]
fn builder_starts_empty() {
    let query = QueryBuilder::<Photo>::new().build();
    assert!(query.filters.is_empty());
    assert!(query.sorting.is_empty());
    assert!(query.joins.is_empty());
    assert!(query.group_by.is_none());
    assert!(query.paging.is_none());
}

#[test]
fn filter_chaining_preserves_order() {
    let query = QueryBuilder::<Photo>::new()
        .filter(
            "status",
            FilterOperator::Eq,
            Value::String("new".to_string()),
        )
        .filter("size", FilterOperator::Gt, Value::Int(10))
        .build();

    assert_eq!(query.filters.len(), 2);
    assert_eq!(query.filters[0].field, "status");
    assert_eq!(query.filters[1].field, "size");
}

#[test]
fn join_uses_entity_name() {
    let query = QueryBuilder::<Photo>::new()
        .join::<Album>("photo.album_id", "album.id")
        .build();

    assert_eq!(query.joins.len(), 1);
    assert_eq!(query.joins[0].entity_name, "album");
    assert_eq!(query.joins[0].on.len(), 1);
    assert_eq!(query.joins[0].on[0].left, "photo.album_id");
    assert_eq!(query.joins[0].on[0].right, "album.id");
}

#[test]
fn group_by_and_aggregates_added() {
    let query = QueryBuilder::<Photo>::new()
        .group_by(["status"])
        .count()
        .sum("size")
        .build();

    let group_by = query.group_by.expect("group_by");
    assert_eq!(group_by.fields, vec!["status".to_string()]);
    assert_eq!(group_by.aggregates.len(), 2);
    assert_eq!(group_by.aggregates[0].function, AggregateFunction::Count);
    assert_eq!(group_by.aggregates[1].function, AggregateFunction::Sum);
    assert_eq!(group_by.aggregates[1].field, "size");
}

#[test]
fn page_overrides_previous_paging() {
    let query = QueryBuilder::<Photo>::new().page(1, 10).page(2, 25).build();

    assert_eq!(query.paging, Some(PageRequest::new(2, 25)));
}

#[test]
fn build_produces_expected_query() {
    let query = QueryBuilder::<Photo>::new()
        .filter("active", FilterOperator::Eq, Value::Bool(true))
        .sort_asc("created_at")
        .sort_desc("id")
        .page(3, 15)
        .build();

    assert_eq!(query.filters.len(), 1);
    assert_eq!(query.filters[0].field, "active");
    assert_eq!(query.sorting.len(), 2);
    assert_eq!(query.sorting[0].field, "created_at");
    assert_eq!(query.sorting[0].direction, SortDirection::Asc);
    assert_eq!(query.sorting[1].field, "id");
    assert_eq!(query.sorting[1].direction, SortDirection::Desc);
    assert_eq!(query.paging, Some(PageRequest::new(3, 15)));
}
