use sqlx::PgPool;

use crate::{
    application::authorization::ContactsCreateGrant,
    db::models::{audit_event::ContactAuditAction, contact::Contact},
    domain::contact::{ContactError, CreateContactInput},
};

use super::{ContactValidation, audit::commit_audited_contact_mutation};

#[derive(Debug)]
pub(crate) struct CreateContactCommand {
    name: String,
    email: String,
}

impl CreateContactCommand {
    pub(crate) fn new(name: String, email: String) -> Self {
        Self { name, email }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreateContactOutcome {
    Created,
    Validation(ContactValidation),
    EmailAlreadyExists,
}

pub(crate) async fn create_contact(
    pool: &PgPool,
    authorization: ContactsCreateGrant,
    command: CreateContactCommand,
) -> Result<CreateContactOutcome, ContactError> {
    let input = CreateContactInput::from_untrusted(command.name, command.email);

    let input = match input.validate() {
        Ok(input) => input,

        Err(report) => {
            return Ok(CreateContactOutcome::Validation(
                ContactValidation::from_report(&report),
            ));
        }
    };

    let actor_user_id = authorization.actor_user_id();

    let mut transaction = pool.begin().await.map_err(ContactError::storage)?;

    match Contact::create(&mut *transaction, &input).await {
        Ok(contact) => {
            commit_audited_contact_mutation(
                transaction,
                actor_user_id,
                ContactAuditAction::Create,
                contact.id,
            )
            .await?;

            Ok(CreateContactOutcome::Created)
        }

        Err(ContactError::EmailAlreadyExists) => Ok(CreateContactOutcome::EmailAlreadyExists),

        Err(error) => Err(error),
    }
}
