use time::{Duration, OffsetDateTime};
use topcoat::{Result, context::Cx, router::error::RouterErrorExt};
use uuid::Uuid;

use crate::db::models::user_session::UserSession;

use super::session;

const SESSION_REFRESH_INTERVAL_SECONDS: i64 = 15 * 60;

/// Request-scoped authenticated application identity.
///
/// Deliberately does not expose Topcoat's token, token hash, or the persisted
/// UserSession row to pages, components, shards, or procedures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AuthenticatedIdentity {
    user_id: Uuid,
}

impl AuthenticatedIdentity {
    pub(crate) fn user_id(self) -> Uuid {
        self.user_id
    }
}

/// Resolve the authenticated identity for this request, if one exists.
///
/// Session validation remains delegated to the session adapter. Storage or
/// framework failures propagate as server errors rather than being disguised
/// as authentication failures.
pub(crate) async fn current_identity(cx: &Cx) -> Result<Option<AuthenticatedIdentity>> {
    let Some(mut active_session) = session::current(cx).await? else {
        return Ok(None);
    };

    let now = OffsetDateTime::now_utc();

    if should_refresh(&active_session, now) {
        let Some(refreshed_session) = session::refresh(cx).await? else {
            return Ok(None);
        };

        active_session = refreshed_session;
    }

    Ok(Some(AuthenticatedIdentity {
        user_id: active_session.user_id(),
    }))
}

/// Require a valid authenticated identity for the current request.
///
/// A genuinely absent/expired session becomes Topcoat's 401 response.
/// Session-framework or persistence failures remain server errors.
pub(crate) async fn require_auth(cx: &Cx) -> Result<AuthenticatedIdentity> {
    Ok(current_identity(cx).await?.ok_or_unauthorized()?)
}

/// Avoid rewriting persistent session expiry on every authenticated request.
///
/// Refresh halfway through the 30-minute idle window. The existing session
/// adapter independently caps every refresh at the original 8-hour absolute
/// deadline.
fn should_refresh(session: &UserSession, now: OffsetDateTime) -> bool {
    now >= session.updated_at() + Duration::seconds(SESSION_REFRESH_INTERVAL_SECONDS)
}

#[cfg(test)]
mod tests {
    use time::{Duration, OffsetDateTime};
    use uuid::Uuid;

    use crate::db::models::user_session::UserSession;

    use super::should_refresh;

    fn session_with_updated_at(updated_at: OffsetDateTime) -> UserSession {
        UserSession::for_auth_request_test(
            [7_u8; 32],
            Uuid::nil(),
            updated_at + Duration::minutes(30),
            updated_at,
            updated_at,
        )
    }

    #[test]
    fn session_is_not_refreshed_before_half_idle_window() {
        let updated_at = OffsetDateTime::UNIX_EPOCH;

        let session = session_with_updated_at(updated_at);

        assert!(!should_refresh(
            &session,
            updated_at + Duration::minutes(14) + Duration::seconds(59),
        ));
    }

    #[test]
    fn session_is_refreshed_at_half_idle_window() {
        let updated_at = OffsetDateTime::UNIX_EPOCH;

        let session = session_with_updated_at(updated_at);

        assert!(should_refresh(&session, updated_at + Duration::minutes(15),));
    }
}
