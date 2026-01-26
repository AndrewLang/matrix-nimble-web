use nimble_web::entity::entity::Entity;
use nimble_web::entity::metadata::EntityMetadata;
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct City {
    id: i64,
}

impl Entity for City {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "city"
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Key {
    id: i64,
}

impl Entity for Key {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "key"
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BoxItem {
    id: i64,
}

impl Entity for BoxItem {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "box"
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Dish {
    id: i64,
}

impl Entity for Dish {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "dish"
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Class {
    id: i64,
}

impl Entity for Class {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "class"
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

#[test]
fn default_plural_name_handles_es_suffixes() {
    assert_eq!(BoxItem::plural_name(), "boxes");
    assert_eq!(Dish::plural_name(), "dishes");
    assert_eq!(Class::plural_name(), "classes");
}

#[test]
fn default_plural_name_handles_y_suffixes() {
    assert_eq!(City::plural_name(), "cities");
    assert_eq!(Key::plural_name(), "keys");
}

#[test]
fn entity_metadata_includes_id_type_name() {
    let metadata = EntityMetadata::of::<Photo>();
    assert!(metadata.id_type_name().contains("i64"));
    assert_eq!(metadata.name(), "photo");
    assert_eq!(metadata.plural_name(), "photos");
}
