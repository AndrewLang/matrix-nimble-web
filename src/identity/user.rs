use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::identity::claims::Claims;
use crate::identity::identity::Identity;
use crate::identity::kind::IdentityKind;
use crate::identity::method::AuthMethod;

#[derive(Clone)]
pub struct UserIdentity {
    user_id: String,
    claims: Claims,
}

impl UserIdentity {
    pub fn new(user_id: impl Into<String>, claims: Claims) -> Self {
        Self {
            user_id: user_id.into(),
            claims,
        }
    }

    pub fn id(&self) -> &str {
        &self.user_id
    }

    pub fn claims(&self) -> &Claims {
        &self.claims
    }
}

impl Debug for UserIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("UserIdentity")
            .field("user_id", &self.user_id)
            .finish()
    }
}

impl Identity for UserIdentity {
    fn subject(&self) -> &str {
        &self.user_id
    }

    fn kind(&self) -> IdentityKind {
        IdentityKind::User
    }

    fn auth_method(&self) -> AuthMethod {
        AuthMethod::Bearer
    }

    fn claims(&self) -> &Claims {
        &self.claims
    }
}

#[derive(Clone, Debug)]
pub struct AnonymousIdentity {
    claims: Claims,
}

impl AnonymousIdentity {
    pub fn new() -> Self {
        Self {
            claims: Claims::default(),
        }
    }
}

impl Identity for AnonymousIdentity {
    fn subject(&self) -> &str {
        "anonymous"
    }

    fn kind(&self) -> IdentityKind {
        IdentityKind::Anonymous
    }

    fn auth_method(&self) -> AuthMethod {
        AuthMethod::Anonymous
    }

    fn claims(&self) -> &Claims {
        &self.claims
    }

    fn is_authenticated(&self) -> bool {
        false
    }
}
