use sqlx::PgExecutor;
use uuid::Uuid;

/// Typed durable audit actions currently supported by the Contacts
/// application boundary.
///
/// The resource type itself is fixed by the persistence statement rather than
/// caller-controlled so a Contacts mutation cannot accidentally be recorded
/// as some other resource class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContactAuditAction {
    Create,
    Update,
    Delete,
}

impl ContactAuditAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

/// Append one minimal Contacts audit event using the caller's executor.
///
/// The executor is intentionally generic so this write can share the exact
/// same SQL transaction as the protected Contacts mutation.
pub(crate) async fn record_contact_event<'c>(
    executor: impl PgExecutor<'c>,
    actor_user_id: Uuid,
    action: ContactAuditAction,
    resource_id: Uuid,
) -> Result<(), sqlx::Error> {
    let event_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO public.audit_events (
            id,
            actor_user_id,
            action,
            resource_type,
            resource_id
        )
        VALUES ($1, $2, $3, 'contact', $4)
        "#,
    )
    .bind(event_id)
    .bind(actor_user_id)
    .bind(action.as_str())
    .bind(resource_id)
    .execute(executor)
    .await?;

    Ok(())
}
