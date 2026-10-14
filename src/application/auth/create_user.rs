use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use sqlx::PgPool;

use crate::{
    auth::password::hash_password,
    db::models::user::User,
    domain::user::{CreateUserInput, UserError},
};

const MAX_PASSWORD_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreateUserRole {
    Viewer,
    Operator,
}

impl CreateUserRole {
    /// Parse the trusted operator CLI representation exactly.
    ///
    /// Role names are deliberately case-sensitive. There is no fallback role:
    /// an unknown value must be rejected before persistence.
    pub(crate) fn from_operator_input(value: &str) -> Option<Self> {
        match value {
            "viewer" => Some(Self::Viewer),
            "operator" => Some(Self::Operator),
            _ => None,
        }
    }
}

pub(crate) struct CreateUserCommand {
    email: String,
    role: CreateUserRole,
    password: String,
}

impl CreateUserCommand {
    pub(crate) fn new(email: String, role: CreateUserRole, password: String) -> Self {
        Self {
            email,
            role,
            password,
        }
    }
}

impl Debug for CreateUserCommand {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateUserCommand")
            .field("email", &self.email)
            .field("role", &self.role)
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

    let result = match command.role {
        CreateUserRole::Viewer => User::create_viewer(pool, &input, password_hash).await,

        CreateUserRole::Operator => User::create(pool, &input, password_hash).await,
    };

    match result {
        Ok(_) => Ok(CreateUserOutcome::Created),

        Err(UserError::EmailAlreadyExists) => Ok(CreateUserOutcome::EmailAlreadyExists),

        Err(error) => Err(CreateUserError::Storage(error)),
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::db::models::user::User;

    use super::{CreateUserCommand, CreateUserOutcome, CreateUserRole, create_user};

    #[test]
    fn role_parser_is_exact_and_case_sensitive() {
        assert_eq!(
            CreateUserRole::from_operator_input("viewer",),
            Some(CreateUserRole::Viewer),
        );

        assert_eq!(
            CreateUserRole::from_operator_input("operator",),
            Some(CreateUserRole::Operator),
        );

        assert_eq!(CreateUserRole::from_operator_input("Viewer",), None,);

        assert_eq!(CreateUserRole::from_operator_input("administrator",), None,);

        assert_eq!(CreateUserRole::from_operator_input(""), None,);
    }

    #[test]
    fn command_debug_redacts_password() {
        let command = CreateUserCommand::new(
            "operator@example.com".to_owned(),
            CreateUserRole::Operator,
            "do-not-print-this".to_owned(),
        );

        let debug = format!("{command:?}");

        assert!(debug.contains("operator@example.com"));

        assert!(debug.contains("Operator"));

        assert!(debug.contains("[REDACTED]"));

        assert!(!debug.contains("do-not-print-this"));
    }

    #[sqlx::test]
    async fn rejects_invalid_email(pool: PgPool) {
        let outcome = create_user(
            &pool,
            CreateUserCommand::new(
                "not-an-email".to_owned(),
                CreateUserRole::Viewer,
                "test-password".to_owned(),
            ),
        )
        .await
        .expect("invalid input should not be a backend error");

        assert_eq!(outcome, CreateUserOutcome::InvalidEmail,);
    }

    #[sqlx::test]
    async fn creates_normalized_viewer(pool: PgPool) {
        let outcome = create_user(
            &pool,
            CreateUserCommand::new(
                "  VIEWER@EXAMPLE.COM  ".to_owned(),
                CreateUserRole::Viewer,
                "test-password".to_owned(),
            ),
        )
        .await
        .expect("viewer creation should succeed");

        assert_eq!(outcome, CreateUserOutcome::Created,);

        let credentials = User::find_credentials_by_email(&pool, "viewer@example.com")
            .await
            .expect("credential lookup should succeed")
            .expect("created viewer should exist");

        assert_eq!(credentials.user().email, "viewer@example.com",);

        assert_eq!(credentials.user().role, "viewer",);
    }

    #[sqlx::test]
    async fn creates_operator(pool: PgPool) {
        let outcome = create_user(
            &pool,
            CreateUserCommand::new(
                "operator@example.com".to_owned(),
                CreateUserRole::Operator,
                "test-password".to_owned(),
            ),
        )
        .await
        .expect("operator creation should succeed");

        assert_eq!(outcome, CreateUserOutcome::Created,);

        let credentials = User::find_credentials_by_email(&pool, "operator@example.com")
            .await
            .expect("credential lookup should succeed")
            .expect("created operator should exist");

        assert_eq!(credentials.user().role, "operator",);
    }

    #[sqlx::test]
    async fn maps_duplicate_email(pool: PgPool) {
        let first = create_user(
            &pool,
            CreateUserCommand::new(
                "duplicate@example.com".to_owned(),
                CreateUserRole::Viewer,
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
                CreateUserRole::Viewer,
                "second-test-password".to_owned(),
            ),
        )
        .await
        .expect("duplicate should be an application outcome");

        assert_eq!(second, CreateUserOutcome::EmailAlreadyExists,);
    }
}
