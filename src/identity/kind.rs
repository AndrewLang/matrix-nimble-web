#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityKind {
    User,
    Service,
    Anonymous,
}
