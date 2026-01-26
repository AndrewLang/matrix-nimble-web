use nimble_web::data::paging::PageRequest;
use nimble_web::data::query::{
    Aggregate, AggregateFunction, Filter, FilterOperator, GroupBy, Query, Sort, SortDirection,
    Value,
};
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone)]
struct Widget {
    id: i64,
}

impl Entity for Widget {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "widget"
    }

    fn plural_name() -> String {
        "widgets".to_string()
    }
}

#[test]
fn query_default_state_is_empty() {
    let query = Query::<Widget>::new();
    assert_eq!(query.entity_name(), "widget");
    assert_eq!(query.entity_plural_name(), "widgets");
    assert!(query.filters.is_empty());
    assert!(query.sorting.is_empty());
    assert!(query.joins.is_empty());
    assert!(query.group_by.is_none());
    assert!(query.paging.is_none());
}

#[test]
fn filter_and_sort_can_be_composed() {
    let mut query = Query::<Widget>::new();
    query.filters.push(Filter {
        field: "status".to_string(),
        operator: FilterOperator::Eq,
        value: Value::String("active".to_string()),
    });
    query.sorting.push(Sort {
        field: "created_at".to_string(),
        direction: SortDirection::Desc,
    });
    query.paging = Some(PageRequest::new(2, 50));

    assert_eq!(query.filters.len(), 1);
    assert_eq!(query.sorting.len(), 1);
    assert_eq!(query.paging, Some(PageRequest::new(2, 50)));
}

#[test]
fn group_by_with_aggregates_is_retained() {
    let aggregates = vec![
        Aggregate {
            function: AggregateFunction::Count,
            field: "*".to_string(),
            alias: Some("total".to_string()),
        },
        Aggregate {
            function: AggregateFunction::Sum,
            field: "amount".to_string(),
            alias: Some("sum_amount".to_string()),
        },
    ];

    let group_by = GroupBy {
        fields: vec!["status".to_string()],
        aggregates: aggregates.clone(),
    };

    let mut query = Query::<Widget>::new();
    query.group_by = Some(group_by);

    let group = query.group_by.expect("group_by set");
    assert_eq!(group.fields, vec!["status".to_string()]);
    assert_eq!(group.aggregates, aggregates);
}
