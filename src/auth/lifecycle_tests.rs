use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use sqlx::PgPool;
use topcoat::{
    context::{Cx, CxTestBuilder},
    session::{SessionConfig, SessionState, Token, TokenStore, TokenStoreFuture},
};

use crate::{
    application::auth::load_current_user, config::AuthConfig, db::models::user::User,
    domain::user::CreateUserInput,
};

use super::{AuthManager, current_identity, password::hash_password, session};

#[derive(Clone, Default)]
struct TestTokenStore {
    token: Arc<Mutex<Option<Token>>>,
}

impl TestTokenStore {
    fn has_token(&self) -> bool {
        self.token
            .lock()
            .expect("test token store mutex should not be poisoned")
            .is_some()
    }
}

impl TokenStore for TestTokenStore {
    fn read<'a>(&'a self, _cx: &'a Cx) -> TokenStoreFuture<'a, Option<Token>> {
        let token = self
            .token
            .lock()
            .expect("test token store mutex should not be poisoned")
            .clone();

        Box::pin(async move { Ok(token) })
    }

    fn write<'a>(
        &'a self,
        _cx: &'a Cx,
        token: Token,
        _max_age: Duration,
    ) -> TokenStoreFuture<'a, ()> {
        *self
            .token
            .lock()
            .expect("test token store mutex should not be poisoned") = Some(token);

        Box::pin(async { Ok(()) })
    }

    fn delete<'a>(&'a self, _cx: &'a Cx) -> TokenStoreFuture<'a, ()> {
        *self
            .token
            .lock()
            .expect("test token store mutex should not be poisoned") = None;

        Box::pin(async { Ok(()) })
    }
}

#[sqlx::test]
async fn authenticated_session_round_trip_revokes_client_and_server_state(pool: PgPool) {
    let input = CreateUserInput::from_untrusted("session-lifecycle@example.com")
        .validate()
        .expect("test user email should be valid");

    let password_hash =
        hash_password("correct horse battery staple").expect("test password should hash");

    let user = User::create(&pool, &input, password_hash)
        .await
        .expect("test user should persist");

    let token_store = TestTokenStore::default();

    let session_config = SessionConfig::builder()
        .token_store(token_store.clone())
        .lifetime(Duration::from_secs(30 * 60))
        .build();

    let auth_config = AuthConfig::local_topcoat_for_test();

    let auth = AuthManager::from_config(&auth_config);

    let cx = CxTestBuilder::new()
        .app_context(pool.clone())
        .app_context(session_config)
        .app_context(auth)
        .request_context(SessionState::new())
        .build();

    assert!(
        !token_store.has_token(),
        "test transport should begin without a session token",
    );

    let started = session::start(&cx, user.id)
        .await
        .expect("session should start");

    assert!(
        token_store.has_token(),
        "Topcoat should issue a client token when the session starts",
    );

    let current = session::current(&cx)
        .await
        .expect("session lookup should succeed")
        .expect("started session should be current");

    assert_eq!(current.user_id(), user.id,);

    assert_eq!(current.token_hash(), started.token_hash(),);

    let identity = current_identity(&cx)
        .await
        .expect("identity lookup should succeed")
        .expect("started session should authenticate");

    assert_eq!(identity.user_id(), user.id,);

    let current_user = load_current_user(&pool, identity.user_id())
        .await
        .expect("current-user lookup should succeed")
        .expect("authenticated user should exist");

    assert_eq!(current_user.email(), "session-lifecycle@example.com",);

    assert!(
        session::stop(&cx).await.expect("session should stop"),
        "stopping an active persisted session should report revocation",
    );

    assert!(
        !token_store.has_token(),
        "logout/session stop must discard the client token",
    );

    assert!(
        crate::db::models::user_session::UserSession::find_active(&pool, started.token_hash(),)
            .await
            .expect("session persistence lookup should succeed")
            .is_none(),
        "logout/session stop must delete the persisted session",
    );

    assert!(
        session::current(&cx)
            .await
            .expect("post-logout session lookup should succeed")
            .is_none(),
        "the stopped session must not authenticate again",
    );

    assert!(
        current_identity(&cx)
            .await
            .expect("post-logout identity lookup should succeed")
            .is_none(),
        "the request must no longer have an authenticated identity",
    );
}
