use sqlx::PgPool;
use uuid::Uuid;

use crate::application::authorization::ContactsDeleteGrant;

use crate::{db::models::contact::Contact, domain::contact::ContactError};

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
    _authorization: ContactsDeleteGrant,
    command: DeleteContactCommand,
) -> Result<DeleteContactOutcome, ContactError> {
    let id = match Uuid::parse_str(&command.id) {
        Ok(id) => id,
        Err(_) => return Ok(DeleteContactOutcome::NotFound),
    };

    match Contact::delete(pool, id).await {
        Ok(()) => Ok(DeleteContactOutcome::Deleted),

        Err(ContactError::NotFound) => Ok(DeleteContactOutcome::NotFound),

        Err(error) => Err(error),
    }
}
