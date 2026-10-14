mod app;
mod application;
pub(crate) mod auth;
mod config;
mod db;
pub(crate) mod domain;
pub(crate) mod input;
mod web;

use std::error::Error;

pub type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub async fn run() -> AppResult<()> {
    app::run().await
}

pub async fn run_migrations() -> AppResult<()> {
    let database_config = config::DatabaseConfig::load()?;

    let database = db::connect(&database_config, "atlasyr-migrator").await?;

    db::migrate(&database).await?;

    database.close().await;

    Ok(())
}

pub async fn run_create_user(email: String, role: String, password: String) -> AppResult<()> {
    use std::io::{Error as IoError, ErrorKind};

    use application::auth::{CreateUserCommand, CreateUserOutcome, CreateUserRole, create_user};

    let role = CreateUserRole::from_operator_input(&role).ok_or_else(|| {
        IoError::new(
            ErrorKind::InvalidInput,
            r#"role must be exactly "viewer" or "operator""#,
        )
    })?;

    let database_config = config::DatabaseConfig::load()?;

    let database = db::connect(&database_config, "atlasyr-create-user").await?;

    db::healthcheck(&database).await?;

    let result = create_user(&database, CreateUserCommand::new(email, role, password)).await;

    database.close().await;

    match result? {
        CreateUserOutcome::Created => Ok(()),

        CreateUserOutcome::InvalidEmail => {
            Err(IoError::new(ErrorKind::InvalidInput, "email address is invalid").into())
        }

        CreateUserOutcome::InvalidPassword => Err(IoError::new(
            ErrorKind::InvalidInput,
            "password must not be empty and must be at most 1024 bytes",
        )
        .into()),

        CreateUserOutcome::EmailAlreadyExists => Err(IoError::new(
            ErrorKind::AlreadyExists,
            "a user with this email already exists",
        )
        .into()),
    }
}
