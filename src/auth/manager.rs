use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    net::IpAddr,
};

use sqlx::PgPool;
use time::OffsetDateTime;
use topcoat::session::SessionConfig;
use uuid::Uuid;

use crate::config::{AuthConfig, AuthDriverKind};

use super::driver::local_topcoat::LocalTopcoatDriver;

pub(crate) struct AuthManager {
    driver: AuthDriver,
}

enum AuthDriver {
    LocalTopcoat(LocalTopcoatDriver),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CredentialOutcome {
    Authenticated { user_id: Uuid },
    InvalidCredentials,
    Throttled,
}

#[derive(Debug)]
pub(crate) struct AuthError {
    source: Box<dyn Error + Send + Sync>,
}

impl AuthError {
    pub(super) fn backend(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}

impl Display for AuthError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("authentication backend operation failed")
    }
}

impl Error for AuthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl AuthManager {
    pub(crate) fn from_config(config: &AuthConfig) -> Self {
        let driver = match config.driver() {
            AuthDriverKind::LocalTopcoat => AuthDriver::LocalTopcoat(LocalTopcoatDriver::new()),
        };

        Self { driver }
    }

    /// Session configuration is selected by the active auth driver.
    ///
    /// A future provider therefore replaces its security profile at the
    /// same driver boundary instead of leaving independent auth switches
    /// scattered through application startup.
    pub(crate) fn session_config(&self) -> SessionConfig {
        match &self.driver {
            AuthDriver::LocalTopcoat(driver) => driver.session_config(),
        }
    }

    pub(crate) fn cap_session_expiry(
        &self,
        created_at: OffsetDateTime,
        proposed_expires_at: OffsetDateTime,
    ) -> OffsetDateTime {
        match &self.driver {
            AuthDriver::LocalTopcoat(driver) => {
                driver.cap_session_expiry(created_at, proposed_expires_at)
            }
        }
    }

    pub(crate) fn session_is_absolutely_expired(
        &self,
        created_at: OffsetDateTime,
        now: OffsetDateTime,
    ) -> bool {
        match &self.driver {
            AuthDriver::LocalTopcoat(driver) => {
                driver.session_is_absolutely_expired(created_at, now)
            }
        }
    }

    pub(crate) async fn authenticate(
        &self,
        pool: &PgPool,
        email: String,
        password: String,
        client_ip: Option<IpAddr>,
    ) -> Result<CredentialOutcome, AuthError> {
        match &self.driver {
            AuthDriver::LocalTopcoat(driver) => {
                driver.authenticate(pool, email, password, client_ip).await
            }
        }
    }
}
