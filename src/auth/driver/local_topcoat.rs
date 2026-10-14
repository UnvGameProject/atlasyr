use std::{net::IpAddr, time::Duration as StdDuration};

use sqlx::PgPool;
use time::{Duration as TimeDuration, OffsetDateTime};
use topcoat::session::SessionConfig;

use crate::{
    auth::{
        manager::{AuthError, CredentialOutcome},
        password::{PasswordHash, verify_password},
    },
    db::models::user::User,
};

use super::throttle::{LoginThrottle, ThrottleDecision};

const MAX_LOGIN_EMAIL_CHARS: usize = 320;
const MAX_LOGIN_PASSWORD_BYTES: usize = 1024;

const SESSION_IDLE_TIMEOUT: StdDuration = StdDuration::from_secs(30 * 60);

const SESSION_ABSOLUTE_TIMEOUT_HOURS: i64 = 8;

// Deliberately not secret.
//
// Unknown users are checked against a real Argon2id hash so that ordinary
// nonexistent-account attempts still perform expensive verifier work instead
// of taking an obvious fast path.
//
// Parameters match the application's current Argon2id policy:
// m=19456 KiB, t=2, p=1.
const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$dG9wY29hdC1kdW1teS1zbHQ$vgo83j8a2wOuUHf2nbnAnIdsd5z8PPh0FhXraq8uSfo";

pub(in crate::auth) struct LocalTopcoatDriver {
    login_throttle: LoginThrottle,
}

impl LocalTopcoatDriver {
    pub(in crate::auth) fn new() -> Self {
        Self {
            login_throttle: LoginThrottle::new(),
        }
    }

    /// Build the Topcoat session configuration owned by this auth driver.
    ///
    /// Topcoat provides the token/cookie/session machinery. This driver
    /// deliberately overrides Topcoat's much longer default lifetime with
    /// our 30-minute sliding inactivity window.
    pub(in crate::auth) fn session_config(&self) -> SessionConfig {
        SessionConfig::builder()
            .lifetime(SESSION_IDLE_TIMEOUT)
            .build()
    }

    /// Cap a framework-proposed sliding expiry at the session's absolute
    /// lifetime. The persisted application session remains authoritative.
    pub(in crate::auth) fn cap_session_expiry(
        &self,
        created_at: OffsetDateTime,
        proposed_expires_at: OffsetDateTime,
    ) -> OffsetDateTime {
        let absolute_deadline = absolute_session_deadline(created_at);

        proposed_expires_at.min(absolute_deadline)
    }

    pub(in crate::auth) fn session_is_absolutely_expired(
        &self,
        created_at: OffsetDateTime,
        now: OffsetDateTime,
    ) -> bool {
        now >= absolute_session_deadline(created_at)
    }

    pub(in crate::auth) async fn authenticate(
        &self,
        pool: &PgPool,
        email: String,
        password: String,
        client_ip: Option<IpAddr>,
    ) -> Result<CredentialOutcome, AuthError> {
        let email = email.trim();

        let email_within_limit =
            email.chars().take(MAX_LOGIN_EMAIL_CHARS + 1).count() <= MAX_LOGIN_EMAIL_CHARS;

        let password_within_limit = password.len() <= MAX_LOGIN_PASSWORD_BYTES;

        // Only bounded input may become a limiter key.
        //
        // Oversized identifiers intentionally share one fail-safe bucket
        // instead of allocating arbitrary attacker-controlled keys.
        let account_key = if email_within_limit {
            email.to_ascii_lowercase()
        } else {
            "<invalid-login-identifier>".to_owned()
        };

        if self.login_throttle.check(&account_key, client_ip) == ThrottleDecision::Throttled {
            return Ok(CredentialOutcome::Throttled);
        }

        let credentials = if email_within_limit && password_within_limit {
            User::find_credentials_by_email(pool, email)
                .await
                .map_err(AuthError::backend)?
        } else {
            None
        };

        let (user_id, expected_hash) = match credentials {
            Some(credentials) => (
                Some(credentials.user().id),
                credentials.password_hash().clone(),
            ),

            None => (None, PasswordHash::from_stored(DUMMY_PASSWORD_HASH)),
        };

        // Never feed an arbitrarily large attacker-controlled password
        // into the deliberately expensive Argon2 verifier.
        let password = if password_within_limit {
            password
        } else {
            String::new()
        };

        let password_matches = verify_password_blocking(password, expected_hash).await?;

        match (user_id, password_matches) {
            (Some(user_id), true) => Ok(CredentialOutcome::Authenticated { user_id }),

            _ => Ok(CredentialOutcome::InvalidCredentials),
        }
    }
}

fn absolute_session_deadline(created_at: OffsetDateTime) -> OffsetDateTime {
    created_at + TimeDuration::hours(SESSION_ABSOLUTE_TIMEOUT_HOURS)
}

async fn verify_password_blocking(
    password: String,
    expected: PasswordHash,
) -> Result<bool, AuthError> {
    tokio::task::spawn_blocking(move || verify_password(&password, &expected))
        .await
        .map_err(AuthError::backend)?
        .map_err(AuthError::backend)
}

#[cfg(test)]
mod tests {
    use time::{Duration, OffsetDateTime};

    use super::LocalTopcoatDriver;

    #[test]
    fn session_expiry_is_capped_at_absolute_deadline() {
        let driver = LocalTopcoatDriver::new();

        let created_at = OffsetDateTime::UNIX_EPOCH;

        let proposed = created_at + Duration::hours(12);

        assert_eq!(
            driver.cap_session_expiry(created_at, proposed,),
            created_at + Duration::hours(8),
        );
    }

    #[test]
    fn session_expiry_preserves_earlier_idle_deadline() {
        let driver = LocalTopcoatDriver::new();

        let created_at = OffsetDateTime::UNIX_EPOCH;

        let proposed = created_at + Duration::minutes(30);

        assert_eq!(driver.cap_session_expiry(created_at, proposed,), proposed,);
    }

    #[test]
    fn absolute_session_deadline_is_enforced() {
        let driver = LocalTopcoatDriver::new();

        let created_at = OffsetDateTime::UNIX_EPOCH;

        assert!(!driver.session_is_absolutely_expired(
            created_at,
            created_at + Duration::hours(8) - Duration::seconds(1),
        ));

        assert!(driver.session_is_absolutely_expired(created_at, created_at + Duration::hours(8),));
    }
}
