use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::user::User;

use super::policy::{Permission, UserRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AuthorizationActor {
    user_id: Uuid,
    role: UserRole,
}

impl AuthorizationActor {
    fn new(user_id: Uuid, role: UserRole) -> Self {
        Self { user_id, role }
    }

    #[cfg(test)]
    pub(super) fn for_test(user_id: Uuid, role: UserRole) -> Self {
        Self::new(user_id, role)
    }

    pub(super) fn user_id(self) -> Uuid {
        self.user_id
    }

    pub(super) fn role(self) -> UserRole {
        self.role
    }

    pub(super) fn allows(self, permission: Permission) -> bool {
        self.role.allows(permission)
    }
}

#[derive(Debug)]
pub(super) enum AuthorizationActorError {
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

/// Resolve one authenticated user ID into application authorization state.
///
/// Authentication remains responsible only for proving identity. This boundary
/// separately resolves that identity into the role used by application policy.
///
/// A missing user remains distinguishable from storage failure. An unexpected
/// persisted role fails closed instead of silently receiving permissions.
pub(super) async fn load_authorization_actor(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<AuthorizationActor>, AuthorizationActorError> {
    let Some(user) = User::find_by_id(pool, user_id).await? else {
        return Ok(None);
    };

    let role = UserRole::from_persisted(&user.role)
        .map_err(|_| AuthorizationActorError::InvalidPersistedRole)?;

    Ok(Some(AuthorizationActor::new(user.id, role)))
}
