use nimble_web::entity::entity::Entity;
use nimble_web::entity::registry::EntityRegistry;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Photo {
    id: i64,
    name: String,
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
#[allow(dead_code)]
struct Album {
    id: i64,
    title: String,
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
fn basic_entity_definition_exposes_names() {
    assert_eq!(Photo::name(), "photo");
    assert_eq!(Photo::plural_name(), "photos");
}

#[test]
fn entity_id_type_is_accessible() {
    let photo = Photo {
        id: 42,
        name: "Sunset".to_string(),
    };
    let id: &<Photo as Entity>::Id = photo.id();
    assert_eq!(*id, 42);
}

#[test]
fn entity_registry_lookup_returns_metadata() {
    let mut registry = EntityRegistry::new();
    registry.register::<Photo>();

    let metadata = registry.get("photo").expect("photo metadata");

    assert_eq!(metadata.name(), "photo");
    assert_eq!(metadata.plural_name(), "photos");
}

#[test]
fn registry_handles_multiple_entities() {
    let mut registry = EntityRegistry::new();
    registry.register::<Photo>();
    registry.register::<Album>();

    let photo = registry.get("photo").expect("photo metadata");
    let album = registry.get("album").expect("album metadata");

    assert_eq!(photo.name(), "photo");
    assert_eq!(album.name(), "album");
}
