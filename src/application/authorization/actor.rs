use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::user::User;

use super::policy::{Permission, UserRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AuthorizationActor {
    user_id: Uuid,
    role: UserRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContactsReadGrant {
    _private: (),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContactsCreateGrant {
    actor_user_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContactsUpdateGrant {
    actor_user_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContactsDeleteGrant {
    actor_user_id: Uuid,
}

impl AuthorizationActor {
    fn new(user_id: Uuid, role: UserRole) -> Self {
        Self { user_id, role }
    }

    #[cfg(test)]
    pub(super) fn for_test(user_id: Uuid, role: UserRole) -> Self {
        Self::new(user_id, role)
    }

    #[cfg(test)]
    pub(super) fn role(self) -> UserRole {
        self.role
    }

    pub(crate) fn contacts_read(self) -> Option<ContactsReadGrant> {
        self.role
            .allows(Permission::ContactsRead)
            .then_some(ContactsReadGrant { _private: () })
    }

    pub(crate) fn contacts_create(self) -> Option<ContactsCreateGrant> {
        self.role
            .allows(Permission::ContactsCreate)
            .then_some(ContactsCreateGrant {
                actor_user_id: self.user_id,
            })
    }

    pub(crate) fn contacts_update(self) -> Option<ContactsUpdateGrant> {
        self.role
            .allows(Permission::ContactsUpdate)
            .then_some(ContactsUpdateGrant {
                actor_user_id: self.user_id,
            })
    }

    pub(crate) fn contacts_delete(self) -> Option<ContactsDeleteGrant> {
        self.role
            .allows(Permission::ContactsDelete)
            .then_some(ContactsDeleteGrant {
                actor_user_id: self.user_id,
            })
    }
}

impl ContactsCreateGrant {
    pub(crate) const fn actor_user_id(self) -> Uuid {
        self.actor_user_id
    }
}

impl ContactsUpdateGrant {
    pub(crate) const fn actor_user_id(self) -> Uuid {
        self.actor_user_id
    }
}

impl ContactsDeleteGrant {
    pub(crate) const fn actor_user_id(self) -> Uuid {
        self.actor_user_id
    }
}

#[derive(Debug)]
pub(crate) enum AuthorizationActorError {
    Storage(sqlx::Error),
    InvalidPersistedRole,
}

impl Display for AuthorizationActorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(_) => formatter.write_str("authorization actor storage lookup failed"),

            Self::InvalidPersistedRole => {
                formatter.write_str("persisted authorization role is invalid")
            }
        }
    }
}

impl Error for AuthorizationActorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),

            Self::InvalidPersistedRole => None,
        }
    }
}

impl From<sqlx::Error> for AuthorizationActorError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error)
    }
}

/// Resolve an authenticated user ID into application authorization state.
///
/// Authentication proves identity. This boundary independently resolves that
/// identity into the persisted role used by application policy.
///
/// Missing users remain distinguishable from storage failure. Unexpected
/// persisted roles fail closed rather than receiving implicit permissions.
pub(crate) async fn load_authorization_actor(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<AuthorizationActor>, AuthorizationActorError> {
    let Some(user) = User::find_by_id(pool, user_id).await? else {
        return Ok(None);
    };

    let role = UserRole::from_persisted(&user.role)
        .map_err(|_| AuthorizationActorError::InvalidPersistedRole)?;

    Ok(Some(AuthorizationActor::new(user_id, role)))
}
