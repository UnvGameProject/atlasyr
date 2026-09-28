use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::password::hash_password, db::models::user::User, domain::user::CreateUserInput};

use super::{
    actor::{AuthorizationActor, load_authorization_actor},
    policy::{Permission, UserRole},
};

async fn create_test_user(pool: &PgPool, email: &str) -> User {
    let input = CreateUserInput::from_untrusted(email)
        .validate()
        .expect("test user email should be valid");

    let password_hash =
        hash_password("authorization-test-password").expect("test password hashing should succeed");

    User::create(pool, &input, password_hash)
        .await
        .expect("test user should persist")
}

fn actor(role: UserRole) -> AuthorizationActor {
    AuthorizationActor::for_test(role)
}

#[test]
fn operator_can_receive_every_contacts_grant() {
    let actor = actor(UserRole::Operator);

    assert!(actor.contacts_read().is_some());
    assert!(actor.contacts_create().is_some());
    assert!(actor.contacts_update().is_some());
    assert!(actor.contacts_delete().is_some());
}

#[test]
fn viewer_can_receive_read_grant() {
    assert!(actor(UserRole::Viewer).contacts_read().is_some());
}

#[test]
fn viewer_cannot_receive_mutation_grants() {
    let actor = actor(UserRole::Viewer);

    assert!(actor.contacts_create().is_none());
    assert!(actor.contacts_update().is_none());
    assert!(actor.contacts_delete().is_none());
}

#[test]
fn persisted_roles_parse_exactly() {
    assert_eq!(UserRole::from_persisted("operator"), Ok(UserRole::Operator),);

    assert_eq!(UserRole::from_persisted("viewer"), Ok(UserRole::Viewer),);
}

#[test]
fn unknown_persisted_role_fails_closed() {
    assert!(UserRole::from_persisted("administrator").is_err());
    assert!(UserRole::from_persisted("Operator").is_err());
    assert!(UserRole::from_persisted("").is_err());
}

#[test]
fn role_policy_remains_explicit() {
    assert!(UserRole::Operator.allows(Permission::ContactsRead));
    assert!(UserRole::Operator.allows(Permission::ContactsCreate));
    assert!(UserRole::Operator.allows(Permission::ContactsUpdate));
    assert!(UserRole::Operator.allows(Permission::ContactsDelete));

    assert!(UserRole::Viewer.allows(Permission::ContactsRead));
    assert!(!UserRole::Viewer.allows(Permission::ContactsCreate));
    assert!(!UserRole::Viewer.allows(Permission::ContactsUpdate));
    assert!(!UserRole::Viewer.allows(Permission::ContactsDelete));
}

#[sqlx::test]
async fn operator_user_loads_as_operator_actor(pool: PgPool) {
    let user = create_test_user(&pool, "operator-actor@example.com").await;

    let actor = load_authorization_actor(&pool, user.id)
        .await
        .expect("authorization lookup should succeed")
        .expect("authorization actor should exist");

    assert_eq!(actor.role(), UserRole::Operator);

    assert!(actor.contacts_read().is_some());
    assert!(actor.contacts_create().is_some());
    assert!(actor.contacts_update().is_some());
    assert!(actor.contacts_delete().is_some());
}

#[sqlx::test]
async fn viewer_user_loads_as_viewer_actor(pool: PgPool) {
    let user = create_test_user(&pool, "viewer-actor@example.com").await;

    sqlx::query(
        r#"
        UPDATE public.users
        SET role = 'viewer'
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .execute(&pool)
    .await
    .expect("test user role update should succeed");

    let actor = load_authorization_actor(&pool, user.id)
        .await
        .expect("authorization lookup should succeed")
        .expect("authorization actor should exist");

    assert_eq!(actor.role(), UserRole::Viewer);

    assert!(actor.contacts_read().is_some());
    assert!(actor.contacts_create().is_none());
    assert!(actor.contacts_update().is_none());
    assert!(actor.contacts_delete().is_none());
}

#[sqlx::test]
async fn missing_user_has_no_authorization_actor(pool: PgPool) {
    let actor = load_authorization_actor(&pool, Uuid::new_v4())
        .await
        .expect("authorization lookup should succeed");

    assert_eq!(actor, None);
}
