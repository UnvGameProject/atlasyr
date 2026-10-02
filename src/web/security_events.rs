use std::{net::IpAddr, sync::Arc};

#[cfg(test)]
use std::sync::Mutex;

use topcoat::{
    context::Cx,
    router::{
        Body, Layer, LayerFuture, Next, Path,
        error::{ContentTooLargeError, ForbiddenError, InternalServerError, UnauthorizedError},
        request::{client_ip, method},
        try_endpoint,
    },
};
use uuid::Uuid;

/// Stable application-owned security-event categories.
///
/// Events describe security decisions rather than carrying request payloads or
/// arbitrary backend errors. Passwords, account identifiers, cookies, session
/// tokens, token hashes, request bodies, query strings, and arbitrary headers
/// must never become event fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SecurityEventKind {
    AuthenticationSucceeded,
    AuthenticationInvalidCredentials,
    AuthenticationThrottled,
    AuthenticationBackendFailure,
    SessionCreationFailure,
    SessionRevocationFailure,
    SessionEnded,
    RequestUnauthorized,
    RequestForbidden,
    RequestBodyTooLarge,
    RequestInternalError,
}

impl SecurityEventKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AuthenticationSucceeded => "authentication_succeeded",
            Self::AuthenticationInvalidCredentials => "authentication_invalid_credentials",
            Self::AuthenticationThrottled => "authentication_throttled",
            Self::AuthenticationBackendFailure => "authentication_backend_failure",
            Self::SessionCreationFailure => "session_creation_failure",
            Self::SessionRevocationFailure => "session_revocation_failure",
            Self::SessionEnded => "session_ended",
            Self::RequestUnauthorized => "request_unauthorized",
            Self::RequestForbidden => "request_forbidden",
            Self::RequestBodyTooLarge => "request_body_too_large",
            Self::RequestInternalError => "request_internal_error",
        }
    }
}

/// Sanitized security telemetry.
///
/// `route` is Topcoat's matched endpoint pattern rather than the raw URI.
/// Concrete path parameters and query strings therefore do not enter the
/// event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecurityEvent {
    kind: SecurityEventKind,
    method: String,
    route: Option<String>,
    client_ip: Option<IpAddr>,
    user_id: Option<Uuid>,
}

impl SecurityEvent {
    #[cfg(test)]
    pub(crate) const fn kind(&self) -> SecurityEventKind {
        self.kind
    }

    #[cfg(test)]
    pub(crate) fn method(&self) -> &str {
        &self.method
    }

    #[cfg(test)]
    pub(crate) fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    pub(crate) fn log_line(&self) -> String {
        let client_ip = self
            .client_ip
            .map_or_else(|| "-".to_owned(), |ip| ip.to_string());

        let user_id = self
            .user_id
            .map_or_else(|| "-".to_owned(), |id| id.to_string());

        format!(
            "security_event kind={} method={} route={} client_ip={} user_id={}",
            self.kind.as_str(),
            self.method,
            self.route.as_deref().unwrap_or("-"),
            client_ip,
            user_id,
        )
    }
}

trait SecurityEventSink: Send + Sync + 'static {
    fn record(&self, event: &SecurityEvent);
}

#[derive(Debug)]
struct StderrSecurityEventSink;

impl SecurityEventSink for StderrSecurityEventSink {
    fn record(&self, event: &SecurityEvent) {
        eprintln!("{}", event.log_line());
    }
}

/// Application-owned security-event recorder.
///
/// Sink selection is isolated here so later structured tracing or a durable
/// audit exporter can replace stderr without changing authentication,
/// authorization, or router policy code.
#[derive(Clone)]
pub(crate) struct SecurityEventRecorder {
    sink: Arc<dyn SecurityEventSink>,
}

impl SecurityEventRecorder {
    pub(crate) fn stderr() -> Self {
        Self {
            sink: Arc::new(StderrSecurityEventSink),
        }
    }

    pub(crate) fn record_request(&self, cx: &Cx, kind: SecurityEventKind, user_id: Option<Uuid>) {
        let event = SecurityEvent {
            kind,
            method: method(cx).as_str().to_owned(),
            route: try_endpoint(cx).map(|endpoint| endpoint.path().to_string()),
            client_ip: client_ip(cx),
            user_id,
        };

        self.sink.record(&event);
    }

    #[cfg(test)]
    pub(crate) fn capture() -> (Self, SecurityEventCapture) {
        let events = Arc::new(Mutex::new(Vec::new()));

        let recorder = Self {
            sink: Arc::new(CaptureSecurityEventSink {
                events: Arc::clone(&events),
            }),
        };

        let capture = SecurityEventCapture { events };

        (recorder, capture)
    }
}

/// Observe typed router/application errors without converting them.
///
/// Topcoat's `OriginPolicy` executes outside every registered application
/// layer. Origin-policy 403 responses therefore cannot reach this observer.
/// Do not duplicate Topcoat's origin policy or infer its rejection from an
/// unrelated application-level 403.
#[derive(Clone)]
pub(crate) struct SecurityEventLayer {
    recorder: SecurityEventRecorder,
}

impl SecurityEventLayer {
    pub(crate) fn new(recorder: SecurityEventRecorder) -> Self {
        Self { recorder }
    }
}

impl Layer for SecurityEventLayer {
    fn path(&self) -> Option<&Path> {
        None
    }

    fn handle<'a>(&'a self, cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
        Box::pin(async move {
            let result = next.run(cx, body).await;

            if let Err(error) = &result
                && let Some(kind) = classify_router_error(error)
            {
                self.recorder.record_request(cx, kind, None);
            }

            result
        })
    }
}

fn classify_router_error(error: &topcoat::Error) -> Option<SecurityEventKind> {
    if error.downcast_ref::<UnauthorizedError>().is_some() {
        Some(SecurityEventKind::RequestUnauthorized)
    } else if error.downcast_ref::<ForbiddenError>().is_some() {
        Some(SecurityEventKind::RequestForbidden)
    } else if error.downcast_ref::<ContentTooLargeError>().is_some() {
        Some(SecurityEventKind::RequestBodyTooLarge)
    } else if error.downcast_ref::<InternalServerError>().is_some() {
        Some(SecurityEventKind::RequestInternalError)
    } else {
        None
    }
}

#[cfg(test)]
struct CaptureSecurityEventSink {
    events: Arc<Mutex<Vec<SecurityEvent>>>,
}

#[cfg(test)]
impl SecurityEventSink for CaptureSecurityEventSink {
    fn record(&self, event: &SecurityEvent) {
        self.events
            .lock()
            .expect("security-event capture lock should not be poisoned")
            .push(event.clone());
    }
}

#[cfg(test)]
pub(crate) struct SecurityEventCapture {
    events: Arc<Mutex<Vec<SecurityEvent>>>,
}

#[cfg(test)]
impl SecurityEventCapture {
    pub(crate) fn snapshot(&self) -> Vec<SecurityEvent> {
        self.events
            .lock()
            .expect("security-event capture lock should not be poisoned")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use topcoat::router::error::{
        content_too_large, forbidden, internal_server_error, unauthorized,
    };
    use uuid::Uuid;

    use super::{SecurityEvent, SecurityEventKind, classify_router_error};

    #[test]
    fn router_error_classification_is_typed() {
        let unauthorized_error: topcoat::Error = unauthorized().into();
        let forbidden_error: topcoat::Error = forbidden().into();
        let too_large_error: topcoat::Error = content_too_large().into();
        let internal_error: topcoat::Error =
            internal_server_error(std::io::Error::other("do-not-log-internal-source")).into();

        assert_eq!(
            classify_router_error(&unauthorized_error),
            Some(SecurityEventKind::RequestUnauthorized),
        );

        assert_eq!(
            classify_router_error(&forbidden_error),
            Some(SecurityEventKind::RequestForbidden),
        );

        assert_eq!(
            classify_router_error(&too_large_error),
            Some(SecurityEventKind::RequestBodyTooLarge),
        );

        assert_eq!(
            classify_router_error(&internal_error),
            Some(SecurityEventKind::RequestInternalError),
        );
    }

    #[test]
    fn log_line_contains_only_sanitized_fields() {
        let event = SecurityEvent {
            kind: SecurityEventKind::AuthenticationSucceeded,
            method: "POST".to_owned(),
            route: Some("/auth/login".to_owned()),
            client_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10))),
            user_id: Some(Uuid::nil()),
        };

        let line = event.log_line();

        assert!(line.contains("kind=authentication_succeeded"));
        assert!(line.contains("method=POST"));
        assert!(line.contains("route=/auth/login"));
        assert!(line.contains("client_ip=192.0.2.10"));
        assert!(line.contains("user_id=00000000-0000-0000-0000-000000000000"));

        for forbidden in [
            "password=",
            "email=",
            "cookie=",
            "token=",
            "token_hash=",
            "body=",
            "query=",
        ] {
            assert!(
                !line.contains(forbidden),
                "security event contains forbidden field: {line}",
            );
        }
    }
}
