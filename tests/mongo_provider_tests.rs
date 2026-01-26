use mongodb::bson::Bson;
use mongodb::options::ClientOptions;
use mongodb::Client;

use nimble_web::data::mongo::{MongoEntity, MongoProvider};
use nimble_web::data::paging::PageRequest;
use nimble_web::data::provider::{DataError, DataProvider};
use nimble_web::data::query::{Filter, FilterOperator, GroupBy, Join, JoinOn, Query, Value};
use nimble_web::entity::entity::Entity;
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

impl MongoEntity for Photo {
    fn id_field() -> &'static str {
        "id"
    }

    fn id_bson(id: &Self::Id) -> Bson {
        Bson::Int64(*id)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

impl MongoEntity for Album {
    fn id_field() -> &'static str {
        "id"
    }

    fn id_bson(id: &Self::Id) -> Bson {
        Bson::Int64(*id)
    }
}

async fn test_database() -> mongodb::Database {
    let mut options = ClientOptions::parse("mongodb://127.0.0.1:1")
        .await
        .expect("parse options");
    options.server_selection_timeout = Some(Duration::from_millis(200));
    let client = Client::with_options(options).expect("client");
    client.database("nimble_test")
}

#[test]
fn collection_name_uses_plural_name() {
    let name = MongoProvider::<Photo>::collection_name_for_entity(None);
    assert_eq!(name, "photos");
}

#[test]
fn collection_name_includes_schema_prefix() {
    let name = MongoProvider::<Photo>::collection_name_for_entity(Some("tenant"));
    assert_eq!(name, "tenant.photos");
}

#[test]
fn filters_convert_to_bson() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "status".to_string(),
        operator: FilterOperator::Eq,
        value: Value::String("active".to_string()),
    });
    query.filters.push(Filter {
        field: "deleted_at".to_string(),
        operator: FilterOperator::IsNull,
        value: Value::Null,
    });

    let filter = MongoProvider::<Photo>::build_filter_doc(&query).expect("filter");
    let and = filter.get("$and").expect("$and");
    let Bson::Array(items) = and else {
        panic!("expected array");
    };
    assert_eq!(items.len(), 2);
    let first = items[0].as_document().expect("doc 1");
    assert_eq!(
        first.get("status"),
        Some(&Bson::String("active".to_string()))
    );
    let second = items[1].as_document().expect("doc 2");
    assert_eq!(second.get("deleted_at"), Some(&Bson::Null));
}

#[test]
fn numeric_and_binary_filters_convert() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "enabled".to_string(),
        operator: FilterOperator::Eq,
        value: Value::Bool(true),
    });
    query.filters.push(Filter {
        field: "count".to_string(),
        operator: FilterOperator::Gte,
        value: Value::UInt(7),
    });
    query.filters.push(Filter {
        field: "ratio".to_string(),
        operator: FilterOperator::Lt,
        value: Value::Float(1.5),
    });
    query.filters.push(Filter {
        field: "blob".to_string(),
        operator: FilterOperator::Eq,
        value: Value::Bytes(vec![1, 2, 3]),
    });

    let filter = MongoProvider::<Photo>::build_filter_doc(&query).expect("filter");
    let Bson::Array(items) = filter.get("$and").expect("$and") else {
        panic!("expected array");
    };
    assert!(matches!(
        items[0].as_document().unwrap().get("enabled"),
        Some(Bson::Boolean(true))
    ));
    let count = items[1]
        .as_document()
        .unwrap()
        .get("count")
        .unwrap()
        .as_document()
        .unwrap();
    assert!(matches!(count.get("$gte"), Some(Bson::Int64(7))));
    let ratio = items[2]
        .as_document()
        .unwrap()
        .get("ratio")
        .unwrap()
        .as_document()
        .unwrap();
    assert!(matches!(ratio.get("$lt"), Some(Bson::Double(_))));
    let blob = items[3].as_document().unwrap().get("blob").unwrap();
    assert!(matches!(blob, Bson::Binary(_)));
}

#[test]
fn paging_produces_skip_and_limit() {
    let mut query = Query::<Photo>::new();
    query.paging = Some(PageRequest::new(2, 10));

    let pipeline = MongoProvider::<Photo>::build_pipeline(&query).expect("pipeline");
    let skip = pipeline
        .iter()
        .find(|stage| stage.get("$skip").is_some())
        .and_then(|stage| stage.get("$skip"));
    let limit = pipeline
        .iter()
        .find(|stage| stage.get("$limit").is_some())
        .and_then(|stage| stage.get("$limit"));

    assert_eq!(skip, Some(&Bson::Int64(10)));
    assert_eq!(limit, Some(&Bson::Int64(10)));
}

#[test]
fn group_by_produces_group_stage() {
    let mut query = Query::<Photo>::new();
    query.group_by = Some(GroupBy {
        fields: vec!["category".to_string()],
        aggregates: Vec::new(),
    });

    let pipeline = MongoProvider::<Photo>::build_pipeline(&query).expect("pipeline");
    let group = pipeline
        .iter()
        .find(|stage| stage.get("$group").is_some())
        .and_then(|stage| stage.get("$group"));

    assert!(group.is_some());
}

#[test]
fn join_produces_lookup() {
    let mut query = Query::<Photo>::new();
    query.joins.push(Join {
        entity_name: Album::name().to_string(),
        alias: None,
        on: vec![JoinOn {
            left: "photo.album_id".to_string(),
            operator: FilterOperator::Eq,
            right: "album.id".to_string(),
        }],
    });

    let pipeline = MongoProvider::<Photo>::build_pipeline(&query).expect("pipeline");
    let lookup = pipeline
        .iter()
        .find(|stage| stage.get("$lookup").is_some())
        .and_then(|stage| stage.get("$lookup"))
        .and_then(Bson::as_document);

    let lookup = lookup.expect("lookup");
    assert_eq!(
        lookup.get("from"),
        Some(&Bson::String(Album::name().to_string()))
    );
}

#[test]
fn invalid_join_returns_error() {
    let mut query = Query::<Photo>::new();
    query.joins.push(Join {
        entity_name: Album::name().to_string(),
        alias: None,
        on: vec![
            JoinOn {
                left: "photo.album_id".to_string(),
                operator: FilterOperator::Eq,
                right: "album.id".to_string(),
            },
            JoinOn {
                left: "photo.owner_id".to_string(),
                operator: FilterOperator::Eq,
                right: "album.owner_id".to_string(),
            },
        ],
    });

    let result = MongoProvider::<Photo>::build_pipeline(&query);
    assert!(result.is_err());
}

#[test]
fn like_and_text_filters_use_regex() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "name".to_string(),
        operator: FilterOperator::Like,
        value: Value::String("%sun%".to_string()),
    });
    query.filters.push(Filter {
        field: "tag".to_string(),
        operator: FilterOperator::Contains,
        value: Value::String("sky".to_string()),
    });
    query.filters.push(Filter {
        field: "prefix".to_string(),
        operator: FilterOperator::StartsWith,
        value: Value::String("pre".to_string()),
    });
    query.filters.push(Filter {
        field: "suffix".to_string(),
        operator: FilterOperator::EndsWith,
        value: Value::String("fix".to_string()),
    });

    let filter = MongoProvider::<Photo>::build_filter_doc(&query).expect("filter");
    let Bson::Array(items) = filter.get("$and").expect("$and") else {
        panic!("expected array");
    };

    let name = items[0].as_document().expect("name doc");
    assert!(matches!(name.get("name"), Some(Bson::RegularExpression(_))));
    let tag = items[1].as_document().expect("tag doc");
    assert!(matches!(tag.get("tag"), Some(Bson::RegularExpression(_))));
    let prefix = items[2].as_document().expect("prefix doc");
    assert!(matches!(
        prefix.get("prefix"),
        Some(Bson::RegularExpression(_))
    ));
    let suffix = items[3].as_document().expect("suffix doc");
    assert!(matches!(
        suffix.get("suffix"),
        Some(Bson::RegularExpression(_))
    ));
}

#[test]
fn like_escapes_regex_characters() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "name".to_string(),
        operator: FilterOperator::Contains,
        value: Value::String("a+b".to_string()),
    });

    let filter = MongoProvider::<Photo>::build_filter_doc(&query).expect("filter");
    let name_doc = if let Some(Bson::Array(items)) = filter.get("$and") {
        items[0].as_document().expect("name doc")
    } else {
        &filter
    };
    match name_doc.get("name").unwrap() {
        Bson::RegularExpression(regex) => assert_eq!(regex.pattern, "a\\+b"),
        other => panic!("unexpected regex {:?}", other),
    }
}

#[test]
fn list_filters_build_in_and_between() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::In,
        value: Value::List(vec![Value::Int(1), Value::Int(2)]),
    });
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::NotIn,
        value: Value::List(vec![Value::Int(3), Value::Int(4)]),
    });
    query.filters.push(Filter {
        field: "created_at".to_string(),
        operator: FilterOperator::Between,
        value: Value::List(vec![Value::Int(10), Value::Int(20)]),
    });

    let filter = MongoProvider::<Photo>::build_filter_doc(&query).expect("filter");
    let Bson::Array(items) = filter.get("$and").expect("$and") else {
        panic!("expected array");
    };

    let in_doc = items[0].as_document().expect("in doc");
    assert!(in_doc
        .get("id")
        .unwrap()
        .as_document()
        .unwrap()
        .contains_key("$in"));
    let nin_doc = items[1].as_document().expect("nin doc");
    assert!(nin_doc
        .get("id")
        .unwrap()
        .as_document()
        .unwrap()
        .contains_key("$nin"));
    let between_doc = items[2].as_document().expect("between doc");
    let range = between_doc
        .get("created_at")
        .unwrap()
        .as_document()
        .unwrap();
    assert!(range.contains_key("$gte"));
    assert!(range.contains_key("$lte"));
}

#[test]
fn null_filters_translate() {
    let mut query = Query::<Photo>::new();
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

    let filter = MongoProvider::<Photo>::build_filter_doc(&query).expect("filter");
    let Bson::Array(items) = filter.get("$and").expect("$and") else {
        panic!("expected array");
    };
    let is_null = items[0].as_document().expect("is null doc");
    assert_eq!(is_null.get("deleted_at"), Some(&Bson::Null));
    let is_not_null = items[1].as_document().expect("is not null doc");
    let inner = is_not_null
        .get("archived_at")
        .unwrap()
        .as_document()
        .unwrap();
    assert!(inner.contains_key("$ne"));
}

#[test]
fn sort_doc_tracks_sorting() {
    let mut query = Query::<Photo>::new();
    query.sorting.push(nimble_web::data::query::Sort {
        field: "created_at".to_string(),
        direction: nimble_web::data::query::SortDirection::Desc,
    });

    let sort = MongoProvider::<Photo>::build_sort_doc(&query);
    let doc = sort
        .get("$sort")
        .and_then(Bson::as_document)
        .expect("sort doc");
    assert_eq!(doc.get("created_at"), Some(&Bson::Int32(-1)));
}

#[test]
fn group_by_empty_fields_uses_null_id() {
    let mut query = Query::<Photo>::new();
    query.group_by = Some(GroupBy {
        fields: Vec::new(),
        aggregates: Vec::new(),
    });

    let pipeline = MongoProvider::<Photo>::build_pipeline(&query).expect("pipeline");
    let group = pipeline
        .iter()
        .find(|stage| stage.get("$group").is_some())
        .and_then(|stage| stage.get("$group"))
        .and_then(Bson::as_document)
        .expect("group");
    assert_eq!(group.get("_id"), Some(&Bson::Null));
}

#[test]
fn invalid_filter_shapes_return_errors() {
    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::In,
        value: Value::String("bad".to_string()),
    });
    let result = MongoProvider::<Photo>::build_filter_doc(&query);
    assert!(matches!(result, Err(DataError::InvalidQuery(_))));

    let mut query = Query::<Photo>::new();
    query.filters.push(Filter {
        field: "id".to_string(),
        operator: FilterOperator::Between,
        value: Value::List(vec![Value::Int(1)]),
    });
    let result = MongoProvider::<Photo>::build_filter_doc(&query);
    assert!(matches!(result, Err(DataError::InvalidQuery(_))));
}

#[tokio::test]
async fn provider_methods_surface_connection_errors() {
    let database = test_database().await;
    let provider = MongoProvider::<Photo>::new(database);
    let entity = Photo { id: 1 };

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
}

#[tokio::test]
async fn query_rejects_invalid_join_operator() {
    let database = test_database().await;
    let provider = MongoProvider::<Photo>::new(database);
    let mut query = Query::<Photo>::new();
    query.joins.push(Join {
        entity_name: Album::name().to_string(),
        alias: None,
        on: vec![JoinOn {
            left: "photo.album_id".to_string(),
            operator: FilterOperator::Gt,
            right: "album.id".to_string(),
        }],
    });

    let result = provider.query(query).await;
    assert!(matches!(result, Err(DataError::InvalidQuery(_))));
}
