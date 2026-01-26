use mongodb::bson::Bson;

use nimble_web::data::mongo::{MongoEntity, MongoProvider};
use nimble_web::data::paging::PageRequest;
use nimble_web::data::query::{Filter, FilterOperator, GroupBy, Join, JoinOn, Query, Value};
use nimble_web::entity::entity::Entity;

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

#[test]
fn collection_name_uses_plural_name() {
    let name = MongoProvider::<Photo>::collection_name_for_entity(None);
    assert_eq!(name, "photos");
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
