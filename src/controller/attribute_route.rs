use crate::endpoint::route::EndpointRoute;

pub struct RegisteredHttpRoute {
    pub build: fn() -> EndpointRoute,
}

inventory::collect!(RegisteredHttpRoute);

pub fn collected_routes() -> impl Iterator<Item = EndpointRoute> {
    inventory::iter::<RegisteredHttpRoute>
        .into_iter()
        .map(|entry| (entry.build)())
}
