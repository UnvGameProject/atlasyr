//! Regression contracts for Atlasyr's consumed Topcoat HTTP surfaces.
//!
//! These tests use the real application RouterBuilder and request pipeline.
//! They do not mock routing, request context, views or authorization.

use sqlx::PgPool;
use topcoat::router::{Body, Router, request::Request, response::Response, to_bytes};

use crate::{
    auth::AuthManager,
    config::{AppEnvironment, AuthConfig},
    web::ViewRenderer,
};

use super::router_builder;

fn application_router(pool: PgPool) -> Router {
    let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

    let views =
        ViewRenderer::new(AppEnvironment::Testing).expect("registered views should initialize");

    router_builder(pool, auth, views).build()
}

fn get(uri: &str, hx_request: Option<&str>) -> Request {
    let mut builder = Request::builder()
        .method("GET")
        .uri(uri)
        .header("host", "example.test")
        .header("sec-fetch-site", "same-origin");

    if let Some(value) = hx_request {
        builder = builder.header("hx-request", value);
    }

    builder
        .body(Body::empty())
        .expect("test request should build")
}

async fn html_body(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    String::from_utf8(bytes.to_vec()).expect("HTML response should contain UTF-8")
}

#[sqlx::test]
async fn htmx_request_context_preserves_fragment_contract(pool: PgPool) {
    let router = application_router(pool);

    for (header, expected, unexpected) in [
        (None, "ordinary request detected", "htmx request detected"),
        (
            Some("true"),
            "htmx request detected",
            "ordinary request detected",
        ),
        (
            Some("false"),
            "ordinary request detected",
            "htmx request detected",
        ),
    ] {
        let response = router.handle(get("/view-dx/demo", header)).await;

        assert_eq!(
            response.status().as_u16(),
            200,
            "fragment route must remain reachable: {header:?}",
        );

        let html = html_body(response).await;

        assert!(
            html.contains("Server-rendered fragment loaded."),
            "MiniJinja fragment was not rendered: {html}",
        );

        assert!(
            html.contains(expected),
            "Topcoat htmx context not reflected: {html}",
        );

        assert!(
            !html.contains(unexpected),
            "Unexpected request classification: {html}",
        );

        assert!(
            !html.contains("<html"),
            "Fragment route must not become a full page: {html}",
        );
    }
}

#[sqlx::test]
async fn htmx_request_cannot_bypass_authentication(pool: PgPool) {
    let router = application_router(pool);

    let response = router.handle(get("/contacts/list", Some("true"))).await;

    assert_eq!(
        response.status().as_u16(),
        401,
        "HX-Request is presentation metadata, not authentication",
    );

    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff"),
        "Security headers must survive htmx authorization denial",
    );
}
