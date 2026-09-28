use sqlx::PgPool;
use uuid::Uuid;

use crate::application::authorization::{
    ContactsCreateGrant, ContactsDeleteGrant, ContactsReadGrant, ContactsUpdateGrant,
};

use crate::{
    application::contacts::{
        ContactListItem, ContactValidation, CreateContactCommand, CreateContactOutcome,
        DeleteContactCommand, DeleteContactOutcome, UpdateContactCommand, UpdateContactOutcome,
        create_contact as create_contact_use_case, delete_contact as delete_contact_use_case,
        find_contact, list_contacts, update_contact as update_contact_use_case,
    },
    domain::contact::ContactError,
    web::view_models::{
        ContactCreateFormView, ContactRowView, ContactsIndexView, ContactsListView,
        ContactsWorkspaceView,
    },
};

const CREATED_MESSAGE: &str = "Contact created successfully.";

const UPDATED_MESSAGE: &str = "Contact updated.";

const VALIDATION_MESSAGE: &str = "Please correct the highlighted fields.";

const CREATE_ERROR_MESSAGE: &str = "The contact could not be created.";

const UPDATE_ERROR_MESSAGE: &str = "The contact could not be updated.";

const NOT_FOUND_MESSAGE: &str = "This contact is no longer available.";

const NAME_ERROR: &str = "Name is required and must be 200 characters or fewer.";

const EMAIL_ERROR: &str = "Enter a valid email address no longer than 320 characters.";

const DUPLICATE_EMAIL_ERROR: &str = "A contact with this email address already exists.";

const DELETE_ERROR_MESSAGE: &str = "The contact could not be deleted.";

#[derive(Debug, PartialEq, Eq)]
pub(in crate::web) enum ContactDeleteResult {
    Deleted,
    Row(ContactRowView),
}

pub(in crate::web) enum ContactCreateResult {
    Created(ContactsWorkspaceView),
    Render(ContactsWorkspaceView),
}

impl ContactCreateResult {
    pub(in crate::web) fn succeeded(&self) -> bool {
        matches!(self, Self::Created(_))
    }

    pub(in crate::web) fn workspace(&self) -> &ContactsWorkspaceView {
        match self {
            Self::Created(workspace) | Self::Render(workspace) => workspace,
        }
    }

    pub(in crate::web) fn into_workspace(self) -> ContactsWorkspaceView {
        match self {
            Self::Created(workspace) | Self::Render(workspace) => workspace,
        }
    }
}

pub(in crate::web) enum ContactUpdateResult {
    Updated(ContactRowView),
    Render(ContactRowView),
}

impl ContactUpdateResult {
    pub(in crate::web) fn succeeded(&self) -> bool {
        matches!(self, Self::Updated(_))
    }

    pub(in crate::web) fn into_row(self) -> ContactRowView {
        match self {
            Self::Updated(row) | Self::Render(row) => row,
        }
    }
}

pub(in crate::web) async fn index(
    pool: &PgPool,
    read_access: ContactsReadGrant,
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
) -> Result<ContactsIndexView, ContactError> {
    let workspace = workspace(pool, read_access, ContactCreateFormView::empty()).await?;

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

pub(in crate::web) async fn list(
    pool: &PgPool,
    read_access: ContactsReadGrant,
) -> Result<ContactsListView, ContactError> {
    Ok(ContactsListView::new(
        contact_rows(pool, read_access).await?,
    ))
}

pub(in crate::web) async fn row(
    pool: &PgPool,
    read_access: ContactsReadGrant,
    id: &str,
) -> Result<ContactRowView, ContactError> {
    Ok(current_contact_row(pool, read_access, id)
        .await?
        .unwrap_or_else(|| ContactRowView::unavailable(id.to_owned(), NOT_FOUND_MESSAGE)))
}

pub(in crate::web) async fn create(
    pool: &PgPool,
    create_access: ContactsCreateGrant,
    read_access: ContactsReadGrant,
    name: String,
    email: String,
) -> Result<ContactCreateResult, ContactError> {
    let command = CreateContactCommand::new(name.clone(), email.clone());

    let (form, succeeded) = match create_contact_use_case(pool, create_access, command).await {
        Ok(outcome) => {
            let succeeded = matches!(&outcome, CreateContactOutcome::Created);

            (create_form_for_outcome(name, email, outcome), succeeded)
        }

        Err(error) => {
            eprintln!("contact creation failed: {error:?}");

            (
                ContactCreateFormView::new(
                    name,
                    email,
                    None,
                    None,
                    Some(CREATE_ERROR_MESSAGE),
                    None,
                    None,
                ),
                false,
            )
        }
    };

    let workspace = workspace(pool, read_access, form).await?;

    Ok(if succeeded {
        ContactCreateResult::Created(workspace)
    } else {
        ContactCreateResult::Render(workspace)
    })
}

pub(in crate::web) async fn update(
    pool: &PgPool,
    update_access: ContactsUpdateGrant,
    read_access: ContactsReadGrant,
    id: String,
    name: String,
    email: String,
) -> Result<ContactUpdateResult, ContactError> {
    let command = UpdateContactCommand::new(id.clone(), name.clone(), email.clone());

    let outcome = match update_contact_use_case(pool, update_access, command).await {
        Ok(outcome) => outcome,

        Err(error) => {
            eprintln!("contact update failed: {error:?}");

            let current = current_contact_row(pool, read_access, &id).await?;

            let row = current
                .map(|row| row.editing(name, email, None, Some(UPDATE_ERROR_MESSAGE), None, None))
                .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE));

            return Ok(ContactUpdateResult::Render(row));
        }
    };

    let succeeded = matches!(&outcome, UpdateContactOutcome::Updated { .. });

    let current = match outcome {
        UpdateContactOutcome::NotFound => None,

        _ => current_contact_row(pool, read_access, &id).await?,
    };

    let row = update_row_for_outcome(current, id, name, email, outcome);

    Ok(if succeeded {
        ContactUpdateResult::Updated(row)
    } else {
        ContactUpdateResult::Render(row)
    })
}

pub(in crate::web) async fn delete(
    pool: &PgPool,
    delete_access: ContactsDeleteGrant,
    id: String,
    name: String,
    email: String,
) -> Result<ContactDeleteResult, ContactError> {
    let command = DeleteContactCommand::new(id.clone());

    match delete_contact_use_case(pool, delete_access, command).await {
        Ok(outcome) => Ok(delete_result_for_outcome(id, outcome)),

        Err(error) => {
            eprintln!("contact deletion failed: {error:?}");

            Ok(ContactDeleteResult::Row(
                ContactRowView::display(id, name, email).delete_failed(DELETE_ERROR_MESSAGE),
            ))
        }
    }
}

pub(in crate::web) async fn workspace_with_row(
    pool: &PgPool,
    read_access: ContactsReadGrant,
    row: ContactRowView,
) -> Result<ContactsWorkspaceView, ContactError> {
    let row_id = row.id().to_owned();

    let mut contacts = contact_rows(pool, read_access).await?;

    if let Some(position) = contacts.iter().position(|contact| contact.id() == row_id) {
        contacts[position] = row;
    } else {
        contacts.push(row);
    }

    Ok(ContactsWorkspaceView::new(
        ContactCreateFormView::empty(),
        contacts,
    ))
}

async fn workspace(
    pool: &PgPool,
    read_access: ContactsReadGrant,
    form: ContactCreateFormView,
) -> Result<ContactsWorkspaceView, ContactError> {
    Ok(ContactsWorkspaceView::new(
        form,
        contact_rows(pool, read_access).await?,
    ))
}

async fn current_contact_row(
    pool: &PgPool,
    read_access: ContactsReadGrant,
    id: &str,
) -> Result<Option<ContactRowView>, ContactError> {
    let Ok(id) = Uuid::parse_str(id) else {
        return Ok(None);
    };

    Ok(find_contact(pool, read_access, id).await?.map(contact_row))
}

async fn contact_rows(
    pool: &PgPool,
    read_access: ContactsReadGrant,
) -> Result<Vec<ContactRowView>, ContactError> {
    Ok(list_contacts(pool, read_access)
        .await?
        .into_iter()
        .map(contact_row)
        .collect())
}

fn contact_row(contact: ContactListItem) -> ContactRowView {
    ContactRowView::display(contact.id.to_string(), contact.name, contact.email)
}

fn delete_result_for_outcome(id: String, outcome: DeleteContactOutcome) -> ContactDeleteResult {
    match outcome {
        DeleteContactOutcome::Deleted => ContactDeleteResult::Deleted,

        DeleteContactOutcome::NotFound => {
            ContactDeleteResult::Row(ContactRowView::unavailable(id, NOT_FOUND_MESSAGE))
        }
    }
}

fn update_row_for_outcome(
    current: Option<ContactRowView>,
    id: String,
    name: String,
    email: String,
    outcome: UpdateContactOutcome,
) -> ContactRowView {
    match outcome {
        UpdateContactOutcome::Updated { .. } => current
            .map(|row| row.updated(UPDATED_MESSAGE))
            .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE)),

        UpdateContactOutcome::EmailAlreadyExists => current
            .map(|row| {
                row.editing(
                    name,
                    email,
                    Some(VALIDATION_MESSAGE),
                    None,
                    None,
                    Some(DUPLICATE_EMAIL_ERROR),
                )
            })
            .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE)),

        UpdateContactOutcome::Validation(ContactValidation::Name) => current
            .map(|row| {
                row.editing(
                    name,
                    email,
                    Some(VALIDATION_MESSAGE),
                    None,
                    Some(NAME_ERROR),
                    None,
                )
            })
            .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE)),

        UpdateContactOutcome::Validation(ContactValidation::Email) => current
            .map(|row| {
                row.editing(
                    name,
                    email,
                    Some(VALIDATION_MESSAGE),
                    None,
                    None,
                    Some(EMAIL_ERROR),
                )
            })
            .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE)),

        UpdateContactOutcome::Validation(ContactValidation::NameAndEmail) => current
            .map(|row| {
                row.editing(
                    name,
                    email,
                    Some(VALIDATION_MESSAGE),
                    None,
                    Some(NAME_ERROR),
                    Some(EMAIL_ERROR),
                )
            })
            .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE)),

        UpdateContactOutcome::Validation(ContactValidation::General) => current
            .map(|row| row.editing(name, email, Some(VALIDATION_MESSAGE), None, None, None))
            .unwrap_or_else(|| ContactRowView::unavailable(id, NOT_FOUND_MESSAGE)),

        UpdateContactOutcome::NotFound => ContactRowView::unavailable(id, NOT_FOUND_MESSAGE),
    }
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
        application::contacts::{
            ContactValidation, CreateContactOutcome, DeleteContactOutcome, UpdateContactOutcome,
        },
        web::view_models::{ContactCreateFormView, ContactRowView},
    };

    use super::{
        CREATED_MESSAGE, ContactDeleteResult, DUPLICATE_EMAIL_ERROR, EMAIL_ERROR, NAME_ERROR,
        NOT_FOUND_MESSAGE, UPDATED_MESSAGE, VALIDATION_MESSAGE, create_form_for_outcome,
        delete_result_for_outcome, update_row_for_outcome,
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
    fn duplicate_create_outcome_preserves_input_and_marks_email() {
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
    fn both_field_create_validation_preserves_input_and_errors() {
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

    #[test]
    fn updated_outcome_closes_edit_and_reports_success() {
        let current = ContactRowView::display(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada Lovelace".to_owned(),
            "ada@example.com".to_owned(),
        );

        let actual = update_row_for_outcome(
            Some(current),
            "00000000-0000-0000-0000-000000000001".to_owned(),
            " Ada Lovelace ".to_owned(),
            "ADA@example.com".to_owned(),
            UpdateContactOutcome::Updated {
                email: "ada@example.com".to_owned(),
            },
        );

        let expected = ContactRowView::display(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada Lovelace".to_owned(),
            "ada@example.com".to_owned(),
        )
        .updated(UPDATED_MESSAGE);

        assert_eq!(actual, expected);
    }

    #[test]
    fn duplicate_update_preserves_draft_and_marks_email() {
        let current = ContactRowView::display(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada".to_owned(),
            "old@example.com".to_owned(),
        );

        let actual = update_row_for_outcome(
            Some(current),
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada Draft".to_owned(),
            "used@example.com".to_owned(),
            UpdateContactOutcome::EmailAlreadyExists,
        );

        let expected = ContactRowView::display(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada".to_owned(),
            "old@example.com".to_owned(),
        )
        .editing(
            "Ada Draft".to_owned(),
            "used@example.com".to_owned(),
            Some(VALIDATION_MESSAGE),
            None,
            None,
            Some(DUPLICATE_EMAIL_ERROR),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn both_field_update_validation_preserves_draft_and_errors() {
        let current = ContactRowView::display(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada".to_owned(),
            "old@example.com".to_owned(),
        );

        let actual = update_row_for_outcome(
            Some(current),
            "00000000-0000-0000-0000-000000000001".to_owned(),
            String::new(),
            "invalid".to_owned(),
            UpdateContactOutcome::Validation(ContactValidation::NameAndEmail),
        );

        let expected = ContactRowView::display(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Ada".to_owned(),
            "old@example.com".to_owned(),
        )
        .editing(
            String::new(),
            "invalid".to_owned(),
            Some(VALIDATION_MESSAGE),
            None,
            Some(NAME_ERROR),
            Some(EMAIL_ERROR),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn not_found_update_returns_unavailable_row() {
        let id = "00000000-0000-0000-0000-000000000001".to_owned();

        let actual = update_row_for_outcome(
            None,
            id.clone(),
            "Ada".to_owned(),
            "ada@example.com".to_owned(),
            UpdateContactOutcome::NotFound,
        );

        let expected = ContactRowView::unavailable(id, NOT_FOUND_MESSAGE);

        assert_eq!(actual, expected);
    }
    #[test]
    fn deleted_outcome_requests_row_removal() {
        let actual = delete_result_for_outcome(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            DeleteContactOutcome::Deleted,
        );

        assert_eq!(actual, ContactDeleteResult::Deleted,);
    }

    #[test]
    fn not_found_delete_returns_unavailable_row() {
        let id = "00000000-0000-0000-0000-000000000001".to_owned();

        let actual = delete_result_for_outcome(id.clone(), DeleteContactOutcome::NotFound);

        let expected = ContactDeleteResult::Row(ContactRowView::unavailable(id, NOT_FOUND_MESSAGE));

        assert_eq!(actual, expected,);
    }
}
