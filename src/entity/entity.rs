pub trait Entity: Send + Sync + 'static {
    type Id: Send + Sync + Clone + 'static;

    fn id(&self) -> &Self::Id;

    fn name() -> &'static str;
    fn plural_name() -> &'static str;
}
