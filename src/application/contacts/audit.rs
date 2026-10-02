use sqlx::PgTransaction;
use uuid::Uuid;

use crate::{
    db::models::audit_event::{ContactAuditAction, record_contact_event},
    domain::contact::ContactError,
};

/// Complete one successful Contacts mutation by appending its durable audit
/// event and committing both writes atomically.
///
/// Callers hand ownership of the transaction to this function only after the
/// protected mutation has succeeded. If audit persistence fails, the still
/// uncommitted transaction is dropped and SQLx rolls it back.
pub(super) async fn commit_audited_contact_mutation(
    mut transaction: PgTransaction<'_>,
    actor_user_id: Uuid,
    action: ContactAuditAction,
    resource_id: Uuid,
) -> Result<(), ContactError> {
    record_contact_event(&mut *transaction, actor_user_id, action, resource_id)
        .await
        .map_err(ContactError::storage)?;

    transaction.commit().await.map_err(ContactError::storage)
}
