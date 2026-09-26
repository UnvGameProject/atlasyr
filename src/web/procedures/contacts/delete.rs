use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::procedure,
};

use crate::application::contacts::{
    DeleteContactCommand, DeleteContactOutcome, delete_contact as delete_contact_use_case,
};

#[procedure]
pub(in crate::web) async fn delete_contact(
    cx: &Cx,
    id: String,
) -> Result<std::result::Result<String, String>> {
    let pool = app_context::<PgPool>(cx);
    let command = DeleteContactCommand::new(id);

    match delete_contact_use_case(pool, command).await {
        Ok(DeleteContactOutcome::Deleted) => Ok(Ok("deleted".to_owned())),

        Ok(DeleteContactOutcome::NotFound) => Ok(Err("not_found".to_owned())),

        Err(error) => {
            eprintln!("contact deletion failed: {error:?}");
            Ok(Err("error".to_owned()))
        }
    }
}
