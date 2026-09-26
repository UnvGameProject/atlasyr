use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use garde::Validate;
use sqlx::PgPool;

use crate::{
    auth::password::hash_password,
    db::models::user::User,
    domain::user::{CreateUserInput, UserError},
};

const MAX_PASSWORD_BYTES: usize = 1024;

pub(crate) struct CreateUserCommand {
    email: String,
    password: String,
}

impl CreateUserCommand {
    pub(crate) fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

impl Debug for CreateUserCommand {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateUserCommand")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreateUserOutcome {
    Created,
    InvalidEmail,
    InvalidPassword,
    EmailAlreadyExists,
}

#[derive(Debug)]
pub(crate) enum CreateUserError {
    PasswordHash,
    Storage(UserError),
}

impl Display for CreateUserError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::PasswordHash => formatter.write_str("password hashing failed"),

            Self::Storage(_) => formatter.write_str("user storage operation failed"),
        }
    }
}

impl Error for CreateUserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PasswordHash => None,
            Self::Storage(error) => Some(error),
        }
    }
}

pub(crate) async fn create_user(
    pool: &PgPool,
    command: CreateUserCommand,
) -> Result<CreateUserOutcome, CreateUserError> {
    let input = match CreateUserInput::from_untrusted(&command.email).validate() {
        Ok(input) => input,

        Err(_) => {
            return Ok(CreateUserOutcome::InvalidEmail);
        }
    };

    if command.password.is_empty() || command.password.len() > MAX_PASSWORD_BYTES {
        return Ok(CreateUserOutcome::InvalidPassword);
    }

    let password_hash =
        hash_password(&command.password).map_err(|_| CreateUserError::PasswordHash)?;

    match User::create(pool, &input, password_hash).await {
        Ok(_) => Ok(CreateUserOutcome::Created),

        Err(UserError::EmailAlreadyExists) => Ok(CreateUserOutcome::EmailAlreadyExists),

        Err(error) => Err(CreateUserError::Storage(error)),
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::db::models::user::User;

    use super::{CreateUserCommand, CreateUserOutcome, create_user};

    #[test]
    fn command_debug_redacts_password() {
        let command = CreateUserCommand::new(
            "operator@example.com".to_owned(),
            "do-not-print-this".to_owned(),
        );

        let debug = format!("{command:?}");

        assert!(debug.contains("operator@example.com"));

        assert!(debug.contains("[REDACTED]"));

        assert!(!debug.contains("do-not-print-this"));
    }

    #[sqlx::test]
    async fn rejects_invalid_email(pool: PgPool) {
        let outcome = create_user(
            &pool,
            CreateUserCommand::new("not-an-email".to_owned(), "test-password".to_owned()),
        )
        .await
        .expect("invalid input should not be a backend error");

        assert_eq!(outcome, CreateUserOutcome::InvalidEmail,);
    }

    #[sqlx::test]
    async fn creates_normalized_user(pool: PgPool) {
        let outcome = create_user(
            &pool,
            CreateUserCommand::new(
                "  OPERATOR@EXAMPLE.COM  ".to_owned(),
                "test-password".to_owned(),
            ),
        )
        .await
        .expect("user creation should succeed");

        assert_eq!(outcome, CreateUserOutcome::Created,);

        let credentials = User::find_credentials_by_email(&pool, "operator@example.com")
            .await
            .expect("credential lookup should succeed")
            .expect("created user should exist");

        assert_eq!(credentials.user().email, "operator@example.com",);
    }

    #[sqlx::test]
    async fn maps_duplicate_email(pool: PgPool) {
        let first = create_user(
            &pool,
            CreateUserCommand::new(
                "duplicate@example.com".to_owned(),
                "first-test-password".to_owned(),
            ),
        )
        .await
        .expect("first creation should succeed");

        assert_eq!(first, CreateUserOutcome::Created,);

        let second = create_user(
            &pool,
            CreateUserCommand::new(
                "DUPLICATE@example.com".to_owned(),
                "second-test-password".to_owned(),
            ),
        )
        .await
        .expect("duplicate should be an application outcome");

        assert_eq!(second, CreateUserOutcome::EmailAlreadyExists,);
    }
}
