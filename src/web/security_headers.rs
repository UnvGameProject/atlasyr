use topcoat::{
    context::Cx,
    router::{
        Body, Layer, LayerFuture, Next, Path,
        header::{HeaderName, HeaderValue},
        response::response_headers,
    },
};

/// Application-owned browser response security policy.
///
/// This is a pathless Topcoat layer so the same policy applies to ordinary
/// responses, handler errors, and unmatched routes.
///
/// `unsafe-eval` is deliberately limited to script execution because the
/// pinned Topcoat 0.9 browser runtime compiles server-produced runtime
/// expressions with `new Function(...)`. Do not add `unsafe-inline` to the
/// script policy.
///
/// Bootstrap JavaScript mutates element style attributes for components such
/// as Collapse, so style attributes remain permitted while inline `<style>`
/// elements are still denied by `style-src-elem`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SecurityHeaders;

impl SecurityHeaders {
    pub(crate) const CONTENT_SECURITY_POLICY: &str = concat!(
        "default-src 'self'; ",
        "base-uri 'self'; ",
        "object-src 'none'; ",
        "frame-src 'none'; ",
        "frame-ancestors 'none'; ",
        "form-action 'self'; ",
        "script-src 'self' 'unsafe-eval'; ",
        "script-src-elem 'self'; ",
        "script-src-attr 'none'; ",
        "style-src 'self'; ",
        "style-src-elem 'self'; ",
        "style-src-attr 'unsafe-inline'; ",
        "img-src 'self' data:; ",
        "font-src 'self'; ",
        "connect-src 'self'"
    );

    pub(crate) const X_CONTENT_TYPE_OPTIONS: &str = "nosniff";

    pub(crate) const X_FRAME_OPTIONS: &str = "DENY";

    pub(crate) const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";

    pub(crate) const PERMISSIONS_POLICY: &str = "camera=(), geolocation=(), microphone=()";

    fn queue(cx: &Cx, name: &'static str, value: &'static str) {
        response_headers(cx).append(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
}

impl Layer for SecurityHeaders {
    fn path(&self) -> Option<&Path> {
        None
    }

    fn handle<'a>(&'a self, cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
        Self::queue(cx, "content-security-policy", Self::CONTENT_SECURITY_POLICY);

        Self::queue(cx, "x-content-type-options", Self::X_CONTENT_TYPE_OPTIONS);

        Self::queue(cx, "x-frame-options", Self::X_FRAME_OPTIONS);

        Self::queue(cx, "referrer-policy", Self::REFERRER_POLICY);

        Self::queue(cx, "permissions-policy", Self::PERMISSIONS_POLICY);

        next.run(cx, body)
    }
}
