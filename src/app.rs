use std::error::Error;

use sqlx::PgPool;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{
        BodyLimit, OriginPolicy, Router, RouterBuilder, RouterBuilderDiscoverExt, TrustedProxies,
    },
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

use crate::{
    auth::AuthManager,
    config::{AppEnvironment, AuthConfig, DatabaseConfig, ProxyConfig},
    db,
    web::{
        ViewRenderer,
        security_events::{SecurityEventLayer, SecurityEventRecorder},
        security_headers::SecurityHeaders,
    },
};

// Current application request bodies are small HTML forms and runtime
// procedure payloads. Keep their buffering ceiling explicit and conservative.
// A future upload/import route must opt into a larger scoped BodyLimit rather
// than silently widening this application-wide boundary.
const REQUEST_BODY_LIMIT_BYTES: usize = 64 * 1024;

pub(crate) async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app_environment = AppEnvironment::load()?;

    let database_config = DatabaseConfig::load()?;

    let auth_config = AuthConfig::load()?;

    // Forwarded client addresses are trusted only from explicitly
    // configured proxy networks. With no configuration Topcoat uses
    // the direct peer and ignores client-supplied forwarding headers.
    let proxy_config = ProxyConfig::load()?;

    let database = db::connect(&database_config, "topcoat-poc").await?;

    db::healthcheck(&database).await?;

    let auth = AuthManager::from_config(&auth_config);

    let views = ViewRenderer::new(app_environment)?;

    let trusted_proxies = TrustedProxies::new().networks(proxy_config.trusted_networks());

    let router = router_builder_with_trusted_proxies(database, auth, views, trusted_proxies)
        .assets(AssetBundle::load().expect("Topcoat asset bundle was not found"))
        .build();

    topcoat::start(router).await?;

    Ok(())
}

/// Build the application's real HTTP security/application router stack
/// from already-validated dependencies.
///
/// Production startup and router-level authorization tests share this builder
/// so cookies, sessions, runtime routing, discovery, and application contexts
/// cannot silently diverge. Production adds its filesystem-backed asset bundle
/// after this shared security boundary.
#[cfg(test)]
fn router_builder(database: PgPool, auth: AuthManager, views: ViewRenderer) -> RouterBuilder {
    router_builder_with_trusted_proxies(database, auth, views, TrustedProxies::new())
}

fn router_builder_with_trusted_proxies(
    database: PgPool,
    auth: AuthManager,
    views: ViewRenderer,
    trusted_proxies: TrustedProxies,
) -> RouterBuilder {
    router_builder_with_security_events_and_trusted_proxies(
        database,
        auth,
        views,
        SecurityEventRecorder::stderr(),
        trusted_proxies,
    )
}

#[cfg(test)]
fn router_builder_with_security_events(
    database: PgPool,
    auth: AuthManager,
    views: ViewRenderer,
    security_events: SecurityEventRecorder,
) -> RouterBuilder {
    router_builder_with_security_events_and_trusted_proxies(
        database,
        auth,
        views,
        security_events,
        TrustedProxies::new(),
    )
}

fn router_builder_with_security_events_and_trusted_proxies(
    database: PgPool,
    auth: AuthManager,
    views: ViewRenderer,
    security_events: SecurityEventRecorder,
    trusted_proxies: TrustedProxies,
) -> RouterBuilder {
    // The selected auth driver owns the complete session-security profile.
    // There is no independent runtime switch that can weaken these settings.
    let session_config = auth.session_config();

    Router::builder()
        // Forwarded client identity is resolved only through the
        // explicitly configured trusted-proxy boundary.
        .trusted_proxies(trusted_proxies)
        // Cookie-authenticated browser mutations must remain same-origin.
        // Keep Topcoat's strict policy explicit: no trusted cross-origin
        // peers and no exempt routes.
        .origin_policy(OriginPolicy::new())
        .cookies()
        .sessions(session_config)
        // Topcoat buffering extractors enforce this request-context limit and
        // return 413 before oversized form/procedure bodies reach handlers.
        .layer(BodyLimit::max(REQUEST_BODY_LIMIT_BYTES))
        // Observe typed security-relevant failures emitted by the application
        // layer stack. Topcoat's origin policy runs outside every registered
        // application layer and therefore cannot be observed here.
        .layer(SecurityEventLayer::new(security_events.clone()))
        // Application response security policy must wrap the Topcoat
        // runtime as well as discovered application routes.
        .layer(SecurityHeaders)
        .runtime()
        .discover()
        .app_context(database)
        .app_context(auth)
        .app_context(views)
        .app_context(security_events)
}

#[cfg(test)]
mod tests {
    use topcoat::router::{Body, Router, request::Request, response::Response, to_bytes};

    use crate::{
        application::auth::{CreateUserCommand, CreateUserOutcome, CreateUserRole, create_user},
        auth::AuthManager,
        config::{AppEnvironment, AuthConfig},
        db::models::contact::Contact,
        domain::contact::CreateContactInput,
        web::{
            ViewRenderer,
            security_events::{SecurityEventKind, SecurityEventRecorder},
            security_headers::SecurityHeaders,
        },
    };

    use super::{REQUEST_BODY_LIMIT_BYTES, router_builder, router_builder_with_security_events};

    const VIEWER_EMAIL: &str = "viewer-http-proof@example.com";

    const VIEWER_PASSWORD: &str = "viewer-http-proof-password";

    fn request(
        method: &'static str,
        uri: &str,
        cookie: Option<&str>,
        content_type: Option<&str>,
        body: Body,
    ) -> Request {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("host", "example.test")
            .header("sec-fetch-site", "same-origin");

        if let Some(cookie) = cookie {
            builder = builder.header("cookie", cookie);
        }

        if let Some(content_type) = content_type {
            builder = builder.header("content-type", content_type);
        }

        builder.body(body).expect("test request should build")
    }

    fn cross_origin_browser_post(uri: &str, content_type: &str, body: Body) -> Request {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("host", "example.test")
            .header("origin", "https://attacker.example")
            .header("sec-fetch-site", "cross-site")
            .header("content-type", content_type)
            .body(body)
            .expect("cross-origin test request should build")
    }

    async fn body_text(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        String::from_utf8(bytes.to_vec()).expect("application response should be UTF-8")
    }

    fn active_session_cookie(response: &Response) -> String {
        let set_cookie = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find(|value| value.starts_with("__Host-session=") && !value.contains("Max-Age=0"))
            .expect("successful login should issue an active hardened session cookie");

        assert!(
            set_cookie.contains("Secure"),
            "session cookie must remain Secure: {set_cookie}",
        );

        assert!(
            set_cookie.contains("HttpOnly"),
            "session cookie must remain HttpOnly: {set_cookie}",
        );

        assert!(
            set_cookie.contains("SameSite=Lax"),
            "session cookie must remain SameSite=Lax: {set_cookie}",
        );

        assert!(
            set_cookie.contains("Path=/"),
            "session cookie must remain scoped to /: {set_cookie}",
        );

        set_cookie
            .split(';')
            .next()
            .expect("Set-Cookie must contain name=value")
            .to_owned()
    }

    async fn assert_forbidden(router: &Router, cookie: &str, uri: &str, body: String) {
        let response = router
            .handle(request(
                "POST",
                uri,
                Some(cookie),
                Some("application/x-www-form-urlencoded"),
                Body::from(body),
            ))
            .await;

        assert_eq!(
            response.status().as_u16(),
            403,
            "authenticated viewer mutation must be forbidden at {uri}",
        );
    }

    fn assert_security_headers(response: &Response) {
        let headers = response.headers();

        for (name, expected) in [
            (
                "content-security-policy",
                SecurityHeaders::CONTENT_SECURITY_POLICY,
            ),
            (
                "x-content-type-options",
                SecurityHeaders::X_CONTENT_TYPE_OPTIONS,
            ),
            ("x-frame-options", SecurityHeaders::X_FRAME_OPTIONS),
            ("referrer-policy", SecurityHeaders::REFERRER_POLICY),
            ("permissions-policy", SecurityHeaders::PERMISSIONS_POLICY),
        ] {
            let values: Vec<_> = headers.get_all(name).iter().collect();

            assert_eq!(
                values.len(),
                1,
                "security header must be emitted exactly once: {name}",
            );

            assert_eq!(
                values[0]
                    .to_str()
                    .expect("security header must be valid ASCII"),
                expected,
                "security header value mismatch: {name}",
            );
        }

        assert!(
            headers.get("strict-transport-security").is_none(),
            "HSTS must remain at the verified HTTPS deployment boundary",
        );
    }

    #[sqlx::test]
    async fn security_event_layer_records_typed_failures_without_request_secrets(
        pool: sqlx::PgPool,
    ) {
        let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

        let views = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing view renderer should initialize");

        let (security_events, capture) = SecurityEventRecorder::capture();

        let router =
            router_builder_with_security_events(pool, auth, views, security_events).build();

        let query_secret = "do-not-log-query-value";

        let unauthorized = router
            .handle(request(
                "GET",
                &format!("/contacts/list?probe={query_secret}"),
                None,
                None,
                Body::empty(),
            ))
            .await;

        assert_eq!(
            unauthorized.status().as_u16(),
            401,
            "protected route should reject an unauthenticated request",
        );

        let body_secret = "do-not-log-request-body";

        let mut oversized_body = body_secret.as_bytes().to_vec();
        oversized_body.resize(REQUEST_BODY_LIMIT_BYTES + 1, b'x');

        let oversized = router
            .handle(request(
                "POST",
                "/contacts/create",
                None,
                Some("application/x-www-form-urlencoded"),
                Body::from(oversized_body),
            ))
            .await;

        assert_eq!(
            oversized.status().as_u16(),
            413,
            "oversized form body should be rejected",
        );

        let events = capture.snapshot();

        assert_eq!(
            events.len(),
            2,
            "401 and 413 should each produce one typed security event",
        );

        assert_eq!(events[0].kind(), SecurityEventKind::RequestUnauthorized,);
        assert_eq!(events[0].method(), "GET");
        assert_eq!(events[0].route(), Some("/contacts/list"));

        assert_eq!(events[1].kind(), SecurityEventKind::RequestBodyTooLarge,);
        assert_eq!(events[1].method(), "POST");
        assert_eq!(events[1].route(), Some("/contacts/create"));

        for event in &events {
            let line = event.log_line();

            assert!(
                !line.contains(query_secret),
                "security event must not contain query-string data: {line}",
            );

            assert!(
                !line.contains(body_secret),
                "security event must not contain request-body data: {line}",
            );
        }

        // OriginPolicy runs outside all registered application layers.
        // Its 403 must therefore not be misclassified as an application
        // authorization denial.
        let origin_rejected = router
            .handle(cross_origin_browser_post(
                "/contacts/create",
                "application/x-www-form-urlencoded",
                Body::from("origin-body-must-not-reach-handler"),
            ))
            .await;

        assert_eq!(
            origin_rejected.status().as_u16(),
            403,
            "Topcoat origin policy should continue rejecting cross-origin mutation",
        );

        assert_eq!(
            capture.snapshot().len(),
            2,
            "origin-policy rejection must not be fabricated as an application-layer event",
        );
    }

    #[sqlx::test]
    async fn internal_storage_errors_are_sanitized_and_security_visible(pool: sqlx::PgPool) {
        const QUERY_MARKER: &str = "internal-query-must-not-be-logged";
        const DATABASE_DETAIL_MARKER: &str = "contacts";

        // Authenticate through the real supported transport before damaging
        // this isolated sqlx::test database. The resulting request therefore
        // exercises the same session and authorization boundaries as a real
        // authenticated browser request.
        let outcome = create_user(
            &pool,
            CreateUserCommand::new(
                VIEWER_EMAIL.to_owned(),
                CreateUserRole::Viewer,
                VIEWER_PASSWORD.to_owned(),
            ),
        )
        .await
        .expect("viewer provisioning should succeed");

        assert!(
            matches!(outcome, CreateUserOutcome::Created),
            "viewer provisioning returned an unexpected outcome",
        );

        let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

        let views = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing view renderer should initialize");

        let (security_events, capture) = SecurityEventRecorder::capture();

        let router =
            router_builder_with_security_events(pool.clone(), auth, views, security_events).build();

        let login_response = router
            .handle(request(
                "POST",
                "/auth/login",
                None,
                Some("application/json"),
                Body::from(format!("[\"{}\",\"{}\"]", VIEWER_EMAIL, VIEWER_PASSWORD,)),
            ))
            .await;

        assert_eq!(
            login_response.status().as_u16(),
            200,
            "test viewer should authenticate through the real login procedure",
        );

        let cookie = active_session_cookie(&login_response);

        // Break only Contacts persistence after authentication. Users and
        // persisted sessions remain available, so the request reaches the
        // protected Contacts query and fails there with a real PostgreSQL /
        // SQLx backend error.
        sqlx::query("DROP TABLE contacts")
            .execute(&pool)
            .await
            .expect("isolated test contacts table should be removable");

        let response = router
            .handle(request(
                "GET",
                &format!("/contacts/list?probe={QUERY_MARKER}"),
                Some(&cookie),
                None,
                Body::empty(),
            ))
            .await;

        assert_eq!(
            response.status().as_u16(),
            500,
            "unexpected Contacts storage failure must be a real HTTP 500",
        );

        // Normal Result::Err 500s retain the request context, so the
        // application response-security policy must still be applied.
        assert_security_headers(&response);

        let body = body_text(response).await;

        assert_eq!(
            body, "internal server error",
            "client-facing 500 body must stay generic",
        );

        for forbidden in [
            DATABASE_DETAIL_MARKER,
            "sqlx",
            "postgres",
            "relation",
            "contact storage operation failed",
            VIEWER_EMAIL,
            QUERY_MARKER,
        ] {
            assert!(
                !body
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "internal detail leaked into client 500 body: {body}",
            );
        }

        let events = capture.snapshot();

        let internal_events: Vec<_> = events
            .iter()
            .filter(|event| event.kind() == SecurityEventKind::RequestInternalError)
            .collect();

        assert_eq!(
            internal_events.len(),
            1,
            "the backend failure should emit exactly one request-internal-error event",
        );

        let event = internal_events[0];

        assert_eq!(event.method(), "GET");
        assert_eq!(event.route(), Some("/contacts/list"));

        let line = event.log_line();

        assert!(
            line.contains("kind=request_internal_error"),
            "typed internal-error event missing kind: {line}",
        );

        for forbidden in [
            QUERY_MARKER,
            VIEWER_EMAIL,
            "sqlx",
            "postgres",
            "relation",
            "contact storage operation failed",
        ] {
            assert!(
                !line
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "security event leaked backend/request detail: {line}",
            );
        }
    }

    #[sqlx::test]
    async fn oversized_buffered_requests_are_rejected_before_handlers(pool: sqlx::PgPool) {
        let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

        let views = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing view renderer should initialize");

        let router = router_builder(pool, auth, views).build();

        // Exercise both buffering transports used by the application:
        //
        // - a Topcoat runtime procedure accepting JSON arguments;
        // - an HTML-first contact endpoint using Form<T>.
        //
        // The payload is intentionally malformed for either transport after
        // its first bytes. Size enforcement must win before deserialization,
        // validation, authentication work, or application handler behavior.
        for (uri, content_type) in [
            ("/auth/login", "application/json"),
            ("/contacts/create", "application/x-www-form-urlencoded"),
        ] {
            let response = router
                .handle(request(
                    "POST",
                    uri,
                    None,
                    Some(content_type),
                    Body::from(vec![b'x'; REQUEST_BODY_LIMIT_BYTES + 1]),
                ))
                .await;

            assert_eq!(
                response.status().as_u16(),
                413,
                "oversized buffered request must be rejected at {uri}",
            );

            // The response-security layer remains the outer application
            // boundary even when body extraction fails before the handler.
            assert_security_headers(&response);
        }
    }

    #[sqlx::test]
    async fn security_headers_cover_success_and_router_errors(pool: sqlx::PgPool) {
        let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

        let views = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing view renderer should initialize");

        let router = router_builder(pool, auth, views).build();

        let success = router
            .handle(request("GET", "/view-dx/demo", None, None, Body::from("")))
            .await;

        assert_eq!(
            success.status().as_u16(),
            200,
            "public fragment should remain reachable",
        );

        assert_security_headers(&success);

        let unauthorized = router
            .handle(request("GET", "/contacts/list", None, None, Body::from("")))
            .await;

        assert_eq!(
            unauthorized.status().as_u16(),
            401,
            "protected route should remain unauthorized without a session",
        );

        assert_security_headers(&unauthorized);

        let missing = router
            .handle(request(
                "GET",
                "/__security-header-proof-missing",
                None,
                None,
                Body::from(""),
            ))
            .await;

        assert_eq!(
            missing.status().as_u16(),
            404,
            "unmatched route should remain not found",
        );

        assert_security_headers(&missing);
    }

    #[sqlx::test]
    async fn cross_origin_browser_state_changes_are_rejected_before_handlers(pool: sqlx::PgPool) {
        let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

        let views = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing view renderer should initialize");

        let router = router_builder(pool, auth, views).build();

        for (uri, content_type) in [
            ("/auth/login", "application/json"),
            ("/auth/logout", "application/json"),
            ("/contacts/create", "application/x-www-form-urlencoded"),
            ("/contacts/update", "application/x-www-form-urlencoded"),
            ("/contacts/delete", "application/x-www-form-urlencoded"),
        ] {
            let response = router
                .handle(cross_origin_browser_post(
                    uri,
                    content_type,
                    Body::from("body-must-not-reach-handler"),
                ))
                .await;

            assert_eq!(
                response.status().as_u16(),
                403,
                "cross-origin browser mutation must be rejected before handler execution: {uri}",
            );
        }
    }

    #[sqlx::test]
    async fn authenticated_viewer_can_read_contacts_but_cannot_mutate(pool: sqlx::PgPool) {
        // Provision a real least-privilege account through the supported
        // application provisioning path introduced in POC 4E.
        let outcome = create_user(
            &pool,
            CreateUserCommand::new(
                VIEWER_EMAIL.to_owned(),
                CreateUserRole::Viewer,
                VIEWER_PASSWORD.to_owned(),
            ),
        )
        .await
        .expect("viewer provisioning should succeed");

        assert!(
            matches!(outcome, CreateUserOutcome::Created),
            "viewer provisioning returned an unexpected outcome",
        );

        // Seed one real row so update/delete denial proves an existing record
        // cannot be modified or removed.
        let input =
            CreateContactInput::from_untrusted("Protected Contact", "protected@example.com")
                .validate()
                .expect("contact fixture should validate");

        let contact = Contact::create(&pool, &input)
            .await
            .expect("contact fixture should persist");

        let original = contact.clone();

        assert_eq!(
            Contact::count(&pool)
                .await
                .expect("contact count should succeed",),
            1,
        );

        let auth = AuthManager::from_config(&AuthConfig::local_topcoat_for_test());

        let views = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing view renderer should initialize");

        let router = router_builder(pool.clone(), auth, views).build();

        // Exercise the actual Topcoat procedure HTTP route. Its generated
        // contract is POST + application/json + positional Arguments array.
        let login_response = router
            .handle(request(
                "POST",
                "/auth/login",
                None,
                Some("application/json"),
                Body::from(format!("[\"{}\",\"{}\"]", VIEWER_EMAIL, VIEWER_PASSWORD,)),
            ))
            .await;

        assert_eq!(
            login_response.status().as_u16(),
            200,
            "valid viewer login should succeed",
        );

        let cookie = active_session_cookie(&login_response);

        let login_body = body_text(login_response).await;

        assert_eq!(
            login_body, "\"authenticated\"",
            "login procedure should return its JSON success value",
        );

        // The hardened cookie is replayed as a browser would do on the next
        // same-origin request. Authentication must then resolve the persisted
        // viewer role and permit ContactsRead.
        let contacts_response = router
            .handle(request(
                "GET",
                "/contacts/list",
                Some(&cookie),
                None,
                Body::empty(),
            ))
            .await;

        assert_eq!(
            contacts_response.status().as_u16(),
            200,
            "authenticated viewer should be allowed to read the Contacts list",
        );

        let contacts_html = body_text(contacts_response).await;

        assert!(
            contacts_html.contains("Protected Contact"),
            "viewer should receive persisted contact content",
        );

        assert!(
            contacts_html.contains("protected@example.com"),
            "viewer should receive persisted contact email",
        );

        for forbidden_markup in [
            "action=\"/contacts/update\"",
            "action=\"/contacts/delete\"",
            "Actions",
        ] {
            assert!(
                !contacts_html.contains(forbidden_markup),
                "viewer presentation exposed mutation UI: {forbidden_markup}",
            );
        }

        // A handcrafted request does not gain authority merely because the
        // viewer hides mutation controls.
        assert_forbidden(
            &router,
            &cookie,
            "/contacts/create",
            "name=ViewerCreated&email=viewer-created%40example.com".to_owned(),
        )
        .await;

        assert_eq!(
            Contact::count(&pool)
                .await
                .expect("post-create-denial count should succeed",),
            1,
            "forbidden create must not persist a row",
        );

        assert_forbidden(
            &router,
            &cookie,
            "/contacts/update",
            format!(
                "id={}&name=Tampered&email=tampered%40example.com",
                original.id,
            ),
        )
        .await;

        let after_update = Contact::find_by_id(&pool, original.id)
            .await
            .expect("post-update-denial lookup should succeed")
            .expect("forbidden update must not remove the row");

        assert_eq!(
            after_update, original,
            "forbidden update must not modify the row",
        );

        assert_forbidden(
            &router,
            &cookie,
            "/contacts/delete",
            format!(
                "id={}&name=ProtectedContact&email=protected%40example.com",
                original.id,
            ),
        )
        .await;

        let after_delete = Contact::find_by_id(&pool, original.id)
            .await
            .expect("post-delete-denial lookup should succeed");

        assert_eq!(
            after_delete,
            Some(original),
            "forbidden delete must not remove the row",
        );

        assert_eq!(
            Contact::count(&pool)
                .await
                .expect("final contact count should succeed",),
            1,
            "viewer mutations must leave the database unchanged",
        );
    }
}
