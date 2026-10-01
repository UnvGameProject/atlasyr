use topcoat::{Result, context::Cx, runtime::procedure};

use crate::{
    auth::session,
    web::security_events::{SecurityEventKind, SecurityEventRecorder},
};

/// Revoke the current persisted session and discard its Topcoat client token.
///
/// Logout is intentionally idempotent: callers receive `signed_out` whether
/// or not they arrived with an active session.
#[procedure("/auth/logout")]
pub(in crate::web) async fn logout(cx: &Cx) -> Result<String> {
    let security_events = topcoat::context::app_context::<SecurityEventRecorder>(cx);

    match session::stop(cx).await {
        Ok(true) => {
            security_events.record_request(cx, SecurityEventKind::SessionEnded, None);

            Ok("signed_out".to_owned())
        }

        Ok(false) => Ok("signed_out".to_owned()),

        Err(_) => {
            security_events.record_request(cx, SecurityEventKind::SessionRevocationFailure, None);

            Ok("error".to_owned())
        }
    }
}
