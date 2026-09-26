use topcoat::{Result, context::Cx, runtime::procedure};

use crate::auth::session;

/// Revoke the current persisted session and discard its Topcoat client token.
///
/// Logout is intentionally idempotent: callers receive `signed_out` whether
/// or not they arrived with an active session.
#[procedure("/auth/logout")]
pub(in crate::web) async fn logout(cx: &Cx) -> Result<String> {
    match session::stop(cx).await {
        Ok(_) => Ok("signed_out".to_owned()),

        Err(error) => {
            eprintln!("session revocation failed during logout: {error:?}");

            Ok("error".to_owned())
        }
    }
}
