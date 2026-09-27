use serde::Serialize;

#[derive(Debug, Serialize)]
pub(in crate::web) struct ViewDxDemoView {
    request_kind: &'static str,
    explanation: &'static str,
}

impl ViewDxDemoView {
    pub(in crate::web) fn new(is_htmx_request: bool) -> Self {
        if is_htmx_request {
            Self {
                request_kind: "htmx request detected",
                explanation: "Topcoat read HX-Request from the request context, \
                     then MiniJinja rendered only this HTML fragment.",
            }
        } else {
            Self {
                request_kind: "ordinary request detected",
                explanation: "This public endpoint also works without HX-Request; \
                     the header changes presentation only and is not an \
                     authorization boundary.",
            }
        }
    }
}
