use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::request::client_ip,
    runtime::procedure,
};

use crate::{
    application::auth::{
        AuthenticateUserCommand, AuthenticateUserOutcome,
        authenticate_user as authenticate_user_use_case,
    },
    auth::{AuthManager, session},
};

/// Authenticate credentials and establish a fresh server-backed session.
///
/// Credential verification, throttling, and password work remain inside the
/// configured auth driver. This procedure is only the Topcoat transport
/// adapter.
///
/// A successful authentication explicitly revokes any session already
/// presented by this client before creating a new one. That means a new login
/// begins with a fresh token and a fresh absolute-session lifetime.
#[procedure("/auth/login")]
pub(in crate::web) async fn login(cx: &Cx, email: String, password: String) -> Result<String> {
    let auth = app_context::<AuthManager>(cx);
    let pool = app_context::<PgPool>(cx);

    let command = AuthenticateUserCommand::new(email, password, client_ip(cx));

    let outcome = match authenticate_user_use_case(auth, pool, command).await {
        Ok(outcome) => outcome,

        Err(error) => {
            eprintln!("authentication backend failed: {error:?}");

            return Ok("error".to_owned());
        }
    };

    match outcome {
        AuthenticateUserOutcome::Authenticated { user_id } => {
            // Reauthentication starts a genuinely fresh session. Revoking
            // the previously presented token first also prevents an old
            // authenticated browser session from surviving a user switch.
            if let Err(error) = session::stop(cx).await {
                eprintln!("existing session revocation failed during login: {error:?}");

                return Ok("error".to_owned());
            }

            match session::start(cx, user_id).await {
                Ok(_) => Ok("authenticated".to_owned()),

                Err(error) => {
                    eprintln!("session creation failed after authentication: {error:?}");

                    Ok("error".to_owned())
                }
            }
        }

        AuthenticateUserOutcome::InvalidCredentials => Ok("invalid_credentials".to_owned()),

        AuthenticateUserOutcome::Throttled => {
            // Throttling is independent of account existence, so this
            // communicates backoff without revealing whether the supplied
            // identifier belongs to a user.
            Ok("throttled".to_owned())
        }
    }
}
