use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
};

static MIGRATOR: Migrator = sqlx::migrate!();

pub(crate) async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await?;

    // The runtime application must never be able to alter SQLx's
    // migration history, even though default table privileges are
    // granted to atlasyr_app for application tables.
    sqlx::query(
        "REVOKE ALL PRIVILEGES
         ON TABLE public._sqlx_migrations
         FROM atlasyr_app",
    )
    .execute(pool)
    .await?;

    // Schema migrations intentionally remain independent of deployment-role
    // names because #[sqlx::test] applies them to isolated databases whose
    // PostgreSQL cluster does not define atlasyr_app. The real migration
    // service does define that role, so deployment-specific least privilege
    // is finalized here after every schema migration succeeds.
    //
    // audit_events is append-only from the runtime application's perspective:
    // remove every table privilege inherited through default privileges, then
    // restore INSERT only. Reading/exporting audit history requires a separate
    // explicitly privileged operational path.
    sqlx::query(
        "REVOKE ALL PRIVILEGES
         ON TABLE public.audit_events
         FROM atlasyr_app",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "GRANT INSERT
         ON TABLE public.audit_events
         TO atlasyr_app",
    )
    .execute(pool)
    .await?;

    Ok(())
}
