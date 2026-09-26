mod create;
mod update;
mod validation;

#[cfg(test)]
mod tests;

pub(crate) use create::{CreateContactCommand, CreateContactOutcome, create_contact};
pub(crate) use update::{UpdateContactCommand, UpdateContactOutcome, update_contact};
pub(crate) use validation::ContactValidation;
