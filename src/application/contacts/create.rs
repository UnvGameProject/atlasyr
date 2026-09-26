use sqlx::PgPool;

use crate::{
    db::models::contact::Contact,
    domain::contact::{ContactError, CreateContactInput},
};

use super::ContactValidation;

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

    match Contact::create(pool, &input).await {
        Ok(_) => Ok(CreateContactOutcome::Created),

        Err(ContactError::EmailAlreadyExists) => Ok(CreateContactOutcome::EmailAlreadyExists),

        Err(error) => Err(error),
    }
}
