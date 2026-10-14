use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    application::authorization::ContactsUpdateGrant,
    db::models::{audit_event::ContactAuditAction, contact::Contact},
    domain::contact::{ContactError, UpdateContactInput},
};

use super::{ContactValidation, audit::commit_audited_contact_mutation};

#[derive(Debug)]
pub(crate) struct UpdateContactCommand {
    id: String,
    name: String,
    email: String,
}

impl UpdateContactCommand {
    pub(crate) fn new(id: String, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UpdateContactOutcome {
    Updated { email: String },
    Validation(ContactValidation),
    EmailAlreadyExists,
    NotFound,
}

pub(crate) async fn update_contact(
    pool: &PgPool,
    authorization: ContactsUpdateGrant,
    command: UpdateContactCommand,
) -> Result<UpdateContactOutcome, ContactError> {
    let id = match Uuid::parse_str(&command.id) {
        Ok(id) => id,

        Err(_) => {
            return Ok(UpdateContactOutcome::NotFound);
        }
    };

    let input = UpdateContactInput::from_untrusted(command.name, command.email);

    let input = match input.validate() {
        Ok(input) => input,

        Err(report) => {
            return Ok(UpdateContactOutcome::Validation(
                ContactValidation::from_report(&report),
            ));
        }
    };

    let actor_user_id = authorization.actor_user_id();

    let mut transaction = pool.begin().await.map_err(ContactError::storage)?;

    match Contact::update(&mut *transaction, id, &input).await {
        Ok(contact) => {
            let resource_id = contact.id;
            let email = contact.email;

            commit_audited_contact_mutation(
                transaction,
                actor_user_id,
                ContactAuditAction::Update,
                resource_id,
            )
            .await?;

            Ok(UpdateContactOutcome::Updated { email })
        }

        Err(ContactError::EmailAlreadyExists) => Ok(UpdateContactOutcome::EmailAlreadyExists),

        Err(ContactError::NotFound) => Ok(UpdateContactOutcome::NotFound),

        Err(error) => Err(error),
    }
}
