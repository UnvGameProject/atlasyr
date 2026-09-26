mod create;
mod delete;
mod list;
mod update;
mod validation;

#[cfg(test)]
mod tests;

pub(crate) use create::{CreateContactCommand, CreateContactOutcome, create_contact};
pub(crate) use delete::{DeleteContactCommand, DeleteContactOutcome, delete_contact};
pub(crate) use list::list_contacts;
pub(crate) use update::{UpdateContactCommand, UpdateContactOutcome, update_contact};
pub(crate) use validation::ContactValidation;
