use crate::auth::require_auth;

use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::procedure,
};

use crate::application::contacts::{
    ContactValidation, UpdateContactCommand, UpdateContactOutcome,
    update_contact as update_contact_use_case,
};

#[procedure]
pub(in crate::web) async fn update_contact(
    cx: &Cx,
    id: String,
    name: String,
    email: String,
) -> Result<std::result::Result<String, String>> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);
    let command = UpdateContactCommand::new(id, name, email);

    match update_contact_use_case(pool, command).await {
        Ok(UpdateContactOutcome::Updated { email }) => Ok(Ok(email)),

        Ok(UpdateContactOutcome::EmailAlreadyExists) => Ok(Err("duplicate".to_owned())),

        Ok(UpdateContactOutcome::NotFound) => Ok(Err("not_found".to_owned())),

        Ok(UpdateContactOutcome::Validation(validation)) => {
            let outcome = match validation {
                ContactValidation::Name => "validation_name",
                ContactValidation::Email => "validation_email",
                ContactValidation::NameAndEmail => "validation_both",
                ContactValidation::General => "validation",
            };

            Ok(Err(outcome.to_owned()))
        }

        Err(error) => {
            eprintln!("contact update failed: {error:?}");
            Ok(Err("error".to_owned()))
        }
    }
}
