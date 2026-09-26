use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    net::IpAddr,
};

use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{AuthError, AuthManager, CredentialOutcome};

pub(crate) struct AuthenticateUserCommand {
    email: String,
    password: String,
    client_ip: Option<IpAddr>,
}

impl AuthenticateUserCommand {
    pub(crate) fn new(email: String, password: String, client_ip: Option<IpAddr>) -> Self {
        Self {
            email,
            password,
            client_ip,
        }
    }
}

impl fmt::Debug for AuthenticateUserCommand {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticateUserCommand")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .field("client_ip", &self.client_ip)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthenticateUserOutcome {
    Authenticated { user_id: Uuid },
    InvalidCredentials,
    Throttled,
}

#[derive(Debug)]
pub(crate) struct AuthenticateUserError {
    source: AuthError,
}

impl Display for AuthenticateUserError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("user authentication failed")
    }
}

impl Error for AuthenticateUserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

pub(crate) async fn authenticate_user(
    auth: &AuthManager,
    pool: &PgPool,
    command: AuthenticateUserCommand,
) -> Result<AuthenticateUserOutcome, AuthenticateUserError> {
    let outcome = auth
        .authenticate(pool, command.email, command.password, command.client_ip)
        .await
        .map_err(|source| AuthenticateUserError { source })?;

    Ok(match outcome {
        CredentialOutcome::Authenticated { user_id } => {
            AuthenticateUserOutcome::Authenticated { user_id }
        }

        CredentialOutcome::InvalidCredentials => AuthenticateUserOutcome::InvalidCredentials,

        CredentialOutcome::Throttled => AuthenticateUserOutcome::Throttled,
    })
}
