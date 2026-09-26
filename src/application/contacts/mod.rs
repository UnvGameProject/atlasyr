mod create;
mod validation;

pub(crate) use create::{CreateContactCommand, CreateContactOutcome, create_contact};
pub(crate) use validation::ContactValidation;
