use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::user::User;

/// Presentation-safe account information for the currently authenticated user.
///
/// This is deliberately separate from both the persisted `User` model and the
/// request/session identity. Web presentation does not receive credential or
/// session details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CurrentUser {
    email: String,
}

impl CurrentUser {
    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}

impl From<User> for CurrentUser {
    fn from(user: User) -> Self {
        Self { email: user.email }
    }
}

pub(crate) async fn load_current_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    User::find_by_id(pool, user_id)
        .await
        .map(|user| user.map(CurrentUser::from))
}
