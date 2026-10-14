use std::net::{IpAddr, Ipv4Addr};

use sqlx::PgPool;

use crate::{
    auth::{
        AuthManager,
        password::{PasswordHash, hash_password},
    },
    config::AuthConfig,
    db::models::user::User,
    domain::user::CreateUserInput,
};

use super::{AuthenticateUserCommand, AuthenticateUserOutcome, authenticate_user};

const PASSWORD: &str = "correct horse battery staple";

fn client_ip() -> Option<IpAddr> {
    Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)))
}

fn auth_manager() -> AuthManager {
    AuthManager::from_config(&AuthConfig::local_topcoat_for_test())
}

async fn create_user(pool: &PgPool, email: &str) -> User {
    let input = CreateUserInput::from_untrusted(email)
        .validate()
        .expect("user input should be valid");

    User::create(
        pool,
        &input,
        hash_password(PASSWORD).expect("password should hash"),
    )
    .await
    .expect("user should be created")
}

#[sqlx::test]
async fn authenticates_known_user(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;

    let auth = auth_manager();

    let outcome = authenticate_user(
        &auth,
        &pool,
        AuthenticateUserCommand::new(
            "  ADA@EXAMPLE.COM  ".to_owned(),
            PASSWORD.to_owned(),
            client_ip(),
        ),
    )
    .await
    .expect("authentication should succeed");

    assert_eq!(
        outcome,
        AuthenticateUserOutcome::Authenticated { user_id: user.id },
    );

    Ok(())
}

#[sqlx::test]
async fn wrong_password_returns_invalid_credentials(pool: PgPool) -> Result<(), sqlx::Error> {
    create_user(&pool, "ada@example.com").await;

    let auth = auth_manager();

    let outcome = authenticate_user(
        &auth,
        &pool,
        AuthenticateUserCommand::new(
            "ada@example.com".to_owned(),
            "wrong password".to_owned(),
            client_ip(),
        ),
    )
    .await
    .expect("credential rejection should not be an application error");

    assert_eq!(outcome, AuthenticateUserOutcome::InvalidCredentials,);

    Ok(())
}

#[sqlx::test]
async fn unknown_email_returns_invalid_credentials(pool: PgPool) -> Result<(), sqlx::Error> {
    let auth = auth_manager();

    let outcome = authenticate_user(
        &auth,
        &pool,
        AuthenticateUserCommand::new(
            "missing@example.com".to_owned(),
            PASSWORD.to_owned(),
            client_ip(),
        ),
    )
    .await
    .expect("credential rejection should not be an application error");

    assert_eq!(outcome, AuthenticateUserOutcome::InvalidCredentials,);

    Ok(())
}

#[sqlx::test]
async fn malformed_stored_password_hash_is_an_application_error(
    pool: PgPool,
) -> Result<(), sqlx::Error> {
    let input = CreateUserInput::from_untrusted("broken@example.com")
        .validate()
        .expect("user input should be valid");

    User::create(
        &pool,
        &input,
        PasswordHash::from_stored("not-a-valid-password-hash"),
    )
    .await
    .expect("corrupt credential fixture should be persisted");

    let auth = auth_manager();

    authenticate_user(
        &auth,
        &pool,
        AuthenticateUserCommand::new(
            "broken@example.com".to_owned(),
            PASSWORD.to_owned(),
            client_ip(),
        ),
    )
    .await
    .expect_err("corrupt stored credentials must not look like a normal mismatch");

    Ok(())
}

#[test]
fn authentication_command_debug_redacts_password() {
    let command = AuthenticateUserCommand::new(
        "ada@example.com".to_owned(),
        "super-secret-password".to_owned(),
        client_ip(),
    );

    let debug = format!("{command:?}");

    assert!(debug.contains("ada@example.com"));
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("super-secret-password",));
}
