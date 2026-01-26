use nimble_web::http::request::HttpRequest;
use nimble_web::routing::default_router::DefaultRouter;
use nimble_web::routing::route::Route;
use nimble_web::routing::router::Router;

#[test]
fn static_route_match() {
    let mut router = DefaultRouter::new();
    router.add_route(Route::new("GET", "/photos"));

    let request = HttpRequest::new("GET", "/photos");
    let matched = router.match_request(&request);

    assert!(matched.is_some());
}

#[test]
fn parameter_route_match() {
    let mut router = DefaultRouter::new();
    router.add_route(Route::new("GET", "/photos/{id}"));

    let request = HttpRequest::new("GET", "/photos/123");
    let matched = router.match_request(&request).expect("route match");

    let id = matched.params().get("id").map(String::as_str);
    assert_eq!(id, Some("123"));
}

#[test]
fn method_mismatch_returns_none() {
    let mut router = DefaultRouter::new();
    router.add_route(Route::new("POST", "/photos"));

    let request = HttpRequest::new("GET", "/photos");
    let matched = router.match_request(&request);

    assert!(matched.is_none());
}

#[test]
fn path_mismatch_returns_none() {
    let mut router = DefaultRouter::new();
    router.add_route(Route::new("GET", "/photos"));

    let request = HttpRequest::new("GET", "/albums");
    let matched = router.match_request(&request);

    assert!(matched.is_none());
}

#[test]
fn multiple_routes_match_correct_one() {
    let mut router = DefaultRouter::new();
    router.add_route(Route::new("GET", "/photos"));
    router.add_route(Route::new("GET", "/photos/{id}"));
    router.add_route(Route::new("GET", "/albums/{id}"));

    let request = HttpRequest::new("GET", "/albums/9");
    let matched = router.match_request(&request).expect("route match");

    assert_eq!(matched.route().path(), "/albums/{id}");
    assert_eq!(matched.params().get("id").map(String::as_str), Some("9"));
}

#[test]
fn no_routes_registered_returns_none() {
    let router = DefaultRouter::new();
    let request = HttpRequest::new("GET", "/photos");

    let matched = router.match_request(&request);

    assert!(matched.is_none());
}

#[test]
fn log_routes_does_not_panic() {
    let mut router = DefaultRouter::new();
    router.add_route(Route::new("GET", "/photos"));
    router.log_routes();
}
