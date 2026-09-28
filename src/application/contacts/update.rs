use sqlx::PgPool;
use uuid::Uuid;

use crate::application::authorization::ContactsUpdateGrant;

use crate::{
    db::models::contact::Contact,
    domain::contact::{ContactError, UpdateContactInput},
};

use super::ContactValidation;

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
    _authorization: ContactsUpdateGrant,
    command: UpdateContactCommand,
) -> Result<UpdateContactOutcome, ContactError> {
    let id = match Uuid::parse_str(&command.id) {
        Ok(id) => id,
        Err(_) => return Ok(UpdateContactOutcome::NotFound),
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

    match Contact::update(pool, id, &input).await {
        Ok(contact) => Ok(UpdateContactOutcome::Updated {
            email: contact.email,
        }),

        Err(ContactError::EmailAlreadyExists) => Ok(UpdateContactOutcome::EmailAlreadyExists),

        Err(ContactError::NotFound) => Ok(UpdateContactOutcome::NotFound),

        Err(error) => Err(error),
    }
}
