use std::error::Error;

use sqlx::PgPool;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{OriginPolicy, Router, RouterBuilder, RouterBuilderDiscoverExt},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

use crate::{
    auth::AuthManager,
    config::{AppEnvironment, AuthConfig, DatabaseConfig},
    db,
    web::ViewRenderer,
};

pub(crate) async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app_environment = AppEnvironment::load()?;

    let database_config = DatabaseConfig::load()?;

    let auth_config = AuthConfig::load()?;

    let database = db::connect(&database_config, "topcoat-poc").await?;

    db::healthcheck(&database).await?;

    let auth = AuthManager::from_config(&auth_config);

    let views = ViewRenderer::new(app_environment)?;

    let router = router_builder(database, auth, views)
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
fn router_builder(database: PgPool, auth: AuthManager, views: ViewRenderer) -> RouterBuilder {
    // The selected auth driver owns the complete session-security profile.
    // There is no independent runtime switch that can weaken these settings.
    let session_config = auth.session_config();

    Router::builder()
        // Cookie-authenticated browser mutations must remain same-origin.
        // Keep Topcoat's strict policy explicit: no trusted cross-origin
        // peers and no exempt routes.
        .origin_policy(OriginPolicy::new())
        .cookies()
        .sessions(session_config)
        .runtime()
        .discover()
        .app_context(database)
        .app_context(auth)
        .app_context(views)
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
        web::ViewRenderer,
    };

    use super::router_builder;

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
