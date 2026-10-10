use std::{net::IpAddr, time::Instant};

use topcoat::{
    context::Cx,
    router::{
        Body, Layer, LayerFuture, Next, Path,
        request::{client_ip, method},
        try_endpoint,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestEvent {
    method: String,
    route: Option<String>,
    client_ip: Option<IpAddr>,
    status: Option<u16>,
    outcome: &'static str,
    duration_ms: u128,
}

impl RequestEvent {
    fn log_line(&self) -> String {
        let client_ip = self
            .client_ip
            .map_or_else(|| "-".to_owned(), |ip| ip.to_string());

        let status = self
            .status
            .map_or_else(|| "-".to_owned(), |status| status.to_string());

        format!(
            "request_event method={} route={} client_ip={} status={} outcome={} duration_ms={}",
            self.method,
            self.route.as_deref().unwrap_or("-"),
            client_ip,
            status,
            self.outcome,
            self.duration_ms,
        )
    }
}

/// Sanitized application request telemetry.
///
/// Route values are Topcoat's matched endpoint patterns, never raw URIs.
/// Query strings, request/response bodies, cookies, arbitrary headers,
/// credentials, session material, and backend error values are excluded.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RequestEventLayer;

impl Layer for RequestEventLayer {
    fn path(&self) -> Option<&Path> {
        None
    }

    fn handle<'a>(&'a self, cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
        Box::pin(async move {
            let started = Instant::now();

            let result = next.run(cx, body).await;

            let (status, outcome) = match &result {
                Ok(response) => (Some(response.status().as_u16()), "ok"),

                Err(_) => (None, "error"),
            };

            let event = RequestEvent {
                method: method(cx).as_str().to_owned(),

                route: try_endpoint(cx).map(|endpoint| endpoint.path().to_string()),

                client_ip: client_ip(cx),

                status,

                outcome,

                duration_ms: started.elapsed().as_millis(),
            };

            eprintln!("{}", event.log_line());

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::RequestEvent;

    #[test]
    fn request_event_log_is_bounded_to_operational_fields() {
        let event = RequestEvent {
            method: "GET".to_owned(),

            route: Some("/contacts/{contact_id}/row".to_owned()),

            client_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 9))),

            status: Some(200),

            outcome: "ok",

            duration_ms: 7,
        };

        assert_eq!(
            event.log_line(),
            "request_event method=GET route=/contacts/{contact_id}/row client_ip=192.0.2.9 status=200 outcome=ok duration_ms=7",
        );
    }
}
