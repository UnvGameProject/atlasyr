use sqlx::PgPool;

use crate::{
    application::contacts::{
        ContactValidation, CreateContactCommand, CreateContactOutcome,
        create_contact as create_contact_use_case, list_contacts,
    },
    domain::contact::ContactError,
    web::view_models::{
        ContactCreateFormView, ContactRowView, ContactsIndexView, ContactsListView,
        ContactsWorkspaceView,
    },
};

const CREATED_MESSAGE: &str = "Contact created successfully.";

const VALIDATION_MESSAGE: &str = "Please correct the highlighted fields.";

const CREATE_ERROR_MESSAGE: &str = "The contact could not be created.";

const NAME_ERROR: &str = "Name is required and must be 200 characters or fewer.";

const EMAIL_ERROR: &str = "Enter a valid email address no longer than 320 characters.";

const DUPLICATE_EMAIL_ERROR: &str = "A contact with this email address already exists.";

pub(in crate::web) async fn index(
    pool: &PgPool,
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
) -> Result<ContactsIndexView, ContactError> {
    let workspace = workspace(pool, ContactCreateFormView::empty()).await?;

    Ok(index_with_workspace(
        app_css_url,
        plain_css_url,
        bootstrap_js_url,
        htmx_js_url,
        workspace,
    ))
}

pub(in crate::web) fn index_with_workspace(
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
    workspace: ContactsWorkspaceView,
) -> ContactsIndexView {
    ContactsIndexView::new(
        app_css_url,
        plain_css_url,
        bootstrap_js_url,
        htmx_js_url,
        workspace,
    )
}

pub(in crate::web) async fn list(pool: &PgPool) -> Result<ContactsListView, ContactError> {
    Ok(ContactsListView::new(contact_rows(pool).await?))
}

pub(in crate::web) async fn create(
    pool: &PgPool,
    name: String,
    email: String,
) -> Result<ContactsWorkspaceView, ContactError> {
    let command = CreateContactCommand::new(name.clone(), email.clone());

    let form = match create_contact_use_case(pool, command).await {
        Ok(outcome) => create_form_for_outcome(name, email, outcome),

        Err(error) => {
            eprintln!("contact creation failed: {error:?}");

            ContactCreateFormView::new(
                name,
                email,
                None,
                None,
                Some(CREATE_ERROR_MESSAGE),
                None,
                None,
            )
        }
    };

    workspace(pool, form).await
}

async fn workspace(
    pool: &PgPool,
    form: ContactCreateFormView,
) -> Result<ContactsWorkspaceView, ContactError> {
    Ok(ContactsWorkspaceView::new(form, contact_rows(pool).await?))
}

async fn contact_rows(pool: &PgPool) -> Result<Vec<ContactRowView>, ContactError> {
    Ok(list_contacts(pool)
        .await?
        .into_iter()
        .map(|contact| ContactRowView::new(contact.id.to_string(), contact.name, contact.email))
        .collect())
}

fn create_form_for_outcome(
    name: String,
    email: String,
    outcome: CreateContactOutcome,
) -> ContactCreateFormView {
    match outcome {
        CreateContactOutcome::Created => ContactCreateFormView::new(
            String::new(),
            String::new(),
            Some(CREATED_MESSAGE),
            None,
            None,
            None,
            None,
        ),

        CreateContactOutcome::EmailAlreadyExists => ContactCreateFormView::new(
            name,
            email,
            None,
            Some(VALIDATION_MESSAGE),
            None,
            None,
            Some(DUPLICATE_EMAIL_ERROR),
        ),

        CreateContactOutcome::Validation(ContactValidation::Name) => ContactCreateFormView::new(
            name,
            email,
            None,
            Some(VALIDATION_MESSAGE),
            None,
            Some(NAME_ERROR),
            None,
        ),

        CreateContactOutcome::Validation(ContactValidation::Email) => ContactCreateFormView::new(
            name,
            email,
            None,
            Some(VALIDATION_MESSAGE),
            None,
            None,
            Some(EMAIL_ERROR),
        ),

        CreateContactOutcome::Validation(ContactValidation::NameAndEmail) => {
            ContactCreateFormView::new(
                name,
                email,
                None,
                Some(VALIDATION_MESSAGE),
                None,
                Some(NAME_ERROR),
                Some(EMAIL_ERROR),
            )
        }

        CreateContactOutcome::Validation(ContactValidation::General) => ContactCreateFormView::new(
            name,
            email,
            None,
            Some(VALIDATION_MESSAGE),
            None,
            None,
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::contacts::{ContactValidation, CreateContactOutcome},
        web::view_models::ContactCreateFormView,
    };

    use super::{
        CREATED_MESSAGE, DUPLICATE_EMAIL_ERROR, EMAIL_ERROR, NAME_ERROR, VALIDATION_MESSAGE,
        create_form_for_outcome,
    };

    #[test]
    fn created_outcome_clears_form_and_reports_success() {
        let actual = create_form_for_outcome(
            "Ada Lovelace".to_owned(),
            "ada@example.com".to_owned(),
            CreateContactOutcome::Created,
        );

        let expected = ContactCreateFormView::new(
            String::new(),
            String::new(),
            Some(CREATED_MESSAGE),
            None,
            None,
            None,
            None,
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn duplicate_outcome_preserves_input_and_marks_email() {
        let actual = create_form_for_outcome(
            "Ada Lovelace".to_owned(),
            "ada@example.com".to_owned(),
            CreateContactOutcome::EmailAlreadyExists,
        );

        let expected = ContactCreateFormView::new(
            "Ada Lovelace".to_owned(),
            "ada@example.com".to_owned(),
            None,
            Some(VALIDATION_MESSAGE),
            None,
            None,
            Some(DUPLICATE_EMAIL_ERROR),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn both_field_validation_preserves_input_and_errors() {
        let actual = create_form_for_outcome(
            String::new(),
            "invalid".to_owned(),
            CreateContactOutcome::Validation(ContactValidation::NameAndEmail),
        );

        let expected = ContactCreateFormView::new(
            String::new(),
            "invalid".to_owned(),
            None,
            Some(VALIDATION_MESSAGE),
            None,
            Some(NAME_ERROR),
            Some(EMAIL_ERROR),
        );

        assert_eq!(actual, expected);
    }
}
