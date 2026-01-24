use std::sync::Arc;

use nimble_web::di::ServiceContainer;

#[derive(Debug)]
struct Counter {
    id: u32,
}

#[derive(Debug)]
struct Flag {
    enabled: bool,
}

#[derive(Debug)]
struct Wrapper {
    id: u32,
}

#[test]
fn singleton_returns_same_instance() {
    let mut container = ServiceContainer::new();
    container.register_singleton(|_| Counter { id: 1 });

    let provider = container.build();
    let first = provider.resolve::<Counter>().expect("missing singleton");
    let second = provider.resolve::<Counter>().expect("missing singleton");

    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn scoped_returns_same_in_scope_and_diff_across_scopes() {
    let mut container = ServiceContainer::new();
    container.register_scoped(|_| Counter { id: 2 });

    let provider = container.build();
    let scope_a = provider.create_scope();
    let scope_b = provider.create_scope();

    let a1 = scope_a.resolve::<Counter>().expect("missing scoped");
    let a2 = scope_a.resolve::<Counter>().expect("missing scoped");
    let b1 = scope_b.resolve::<Counter>().expect("missing scoped");

    assert!(Arc::ptr_eq(&a1, &a2));
    assert!(!Arc::ptr_eq(&a1, &b1));
}

#[test]
fn transient_returns_different_instances() {
    let mut container = ServiceContainer::new();
    container.register_transient(|_| Counter { id: 3 });

    let provider = container.build();
    let first = provider.resolve::<Counter>().expect("missing transient");
    let second = provider.resolve::<Counter>().expect("missing transient");

    assert!(!Arc::ptr_eq(&first, &second));
}

#[test]
fn later_registration_overrides_previous() {
    let mut container = ServiceContainer::new();
    container.register_singleton(|_| Counter { id: 10 });
    container.register_singleton(|_| Counter { id: 20 });

    let provider = container.build();
    let resolved = provider.resolve::<Counter>().expect("missing override");

    assert_eq!(resolved.id, 20);
}

#[test]
fn missing_service_returns_none() {
    let container = ServiceContainer::new();
    let provider = container.build();

    assert!(provider.resolve::<Counter>().is_none());
}

#[test]
fn multiple_types_resolve_independently() {
    let mut container = ServiceContainer::new();
    container.register_singleton(|_| Counter { id: 7 });
    container.register_singleton(|_| Flag { enabled: true });

    let provider = container.build();
    let counter = provider.resolve::<Counter>().expect("missing counter");
    let flag = provider.resolve::<Flag>().expect("missing flag");

    assert_eq!(counter.id, 7);
    assert!(flag.enabled);
}

#[test]
fn scoped_isolated_from_root_cache() {
    let mut container = ServiceContainer::new();
    container.register_scoped(|_| Counter { id: 5 });

    let provider = container.build();
    let root_instance = provider.resolve::<Counter>().expect("missing scoped");
    let scope = provider.create_scope();
    let scoped_instance = scope.resolve::<Counter>().expect("missing scoped");

    assert!(!Arc::ptr_eq(&root_instance, &scoped_instance));
}

#[test]
fn transient_ignores_scope_cache() {
    let mut container = ServiceContainer::new();
    container.register_transient(|_| Counter { id: 9 });

    let provider = container.build();
    let scope = provider.create_scope();
    let first = scope.resolve::<Counter>().expect("missing transient");
    let second = scope.resolve::<Counter>().expect("missing transient");

    assert!(!Arc::ptr_eq(&first, &second));
}

#[test]
fn factory_can_resolve_dependency() {
    let mut container = ServiceContainer::new();
    container.register_singleton(|_| Counter { id: 42 });
    container.register_singleton(|provider| {
        let counter = provider.resolve::<Counter>().expect("missing counter");
        Wrapper { id: counter.id }
    });

    let provider = container.build();
    let wrapper = provider.resolve::<Wrapper>().expect("missing wrapper");

    assert_eq!(wrapper.id, 42);
}
