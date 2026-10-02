use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    application::authorization::ContactsDeleteGrant,
    db::models::{audit_event::ContactAuditAction, contact::Contact},
    domain::contact::ContactError,
};

use super::audit::commit_audited_contact_mutation;

#[derive(Debug)]
pub(crate) struct DeleteContactCommand {
    id: String,
}

impl DeleteContactCommand {
    pub(crate) fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeleteContactOutcome {
    Deleted,
    NotFound,
}

pub(crate) async fn delete_contact(
    pool: &PgPool,
    authorization: ContactsDeleteGrant,
    command: DeleteContactCommand,
) -> Result<DeleteContactOutcome, ContactError> {
    let id = match Uuid::parse_str(&command.id) {
        Ok(id) => id,

        Err(_) => {
            return Ok(DeleteContactOutcome::NotFound);
        }
    };

    let actor_user_id = authorization.actor_user_id();

    let mut transaction = pool.begin().await.map_err(ContactError::storage)?;

    match Contact::delete(&mut *transaction, id).await {
        Ok(()) => {
            commit_audited_contact_mutation(
                transaction,
                actor_user_id,
                ContactAuditAction::Delete,
                id,
            )
            .await?;

            Ok(DeleteContactOutcome::Deleted)
        }

        Err(ContactError::NotFound) => Ok(DeleteContactOutcome::NotFound),

        Err(error) => Err(error),
    }
}
