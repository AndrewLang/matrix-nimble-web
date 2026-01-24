# nimble-web

Framework-oriented Rust library scaffold for web applications.

## DI usage

```rust
use nimble_web::di::ServiceContainer;

#[derive(Debug)]
struct Counter {
    id: u32,
}

let mut container = ServiceContainer::new();
container.register_singleton(|_| Counter { id: 1 });

let provider = container.build();
let scope = provider.create_scope();
let counter = scope.resolve::<Counter>().expect("registered");

println!("{}", counter.id);
```

Lifetimes: singleton (one per container), scoped (one per scope), transient (new per resolve).
