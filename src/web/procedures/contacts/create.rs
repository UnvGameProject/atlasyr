use crate::auth::require_auth;

use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::procedure,
};

use crate::application::contacts::{
    ContactValidation, CreateContactCommand, CreateContactOutcome,
    create_contact as create_contact_use_case,
};

#[procedure]
pub(in crate::web) async fn create_contact(cx: &Cx, name: String, email: String) -> Result<String> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);
    let command = CreateContactCommand::new(name, email);

    match create_contact_use_case(pool, command).await {
        Ok(CreateContactOutcome::Created) => Ok("created".to_owned()),

        Ok(CreateContactOutcome::EmailAlreadyExists) => Ok("duplicate".to_owned()),

        Ok(CreateContactOutcome::Validation(validation)) => {
            let outcome = match validation {
                ContactValidation::Name => "validation_name",
                ContactValidation::Email => "validation_email",
                ContactValidation::NameAndEmail => "validation_both",
                ContactValidation::General => "validation",
            };

            Ok(outcome.to_owned())
        }

        Err(error) => {
            eprintln!("contact creation failed: {error:?}");
            Ok("error".to_owned())
        }
    }
}
