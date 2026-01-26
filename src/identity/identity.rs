use std::fmt::Debug;

use crate::identity::claims::Claims;
use crate::identity::kind::IdentityKind;
use crate::identity::method::AuthMethod;

pub trait Identity: Send + Sync + Debug + 'static {
    fn subject(&self) -> &str;
    fn kind(&self) -> IdentityKind;
    fn auth_method(&self) -> AuthMethod;
    fn claims(&self) -> &Claims;
    fn is_authenticated(&self) -> bool {
        true
    }
}
