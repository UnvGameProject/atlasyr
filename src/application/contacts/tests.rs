use crate::domain::contact::CreateContactInput;

use super::ContactValidation;

#[test]
fn contact_validation_identifies_name_error() {
    let input = CreateContactInput::from_untrusted("", "valid@example.com");

    let report = match input.validate() {
        Ok(_) => panic!("expected invalid name"),
        Err(report) => report,
    };

    assert_eq!(
        ContactValidation::from_report(&report),
        ContactValidation::Name,
    );
}

#[test]
fn contact_validation_identifies_email_error() {
    let input = CreateContactInput::from_untrusted("Valid Name", "invalid");

    let report = match input.validate() {
        Ok(_) => panic!("expected invalid email"),
        Err(report) => report,
    };

    assert_eq!(
        ContactValidation::from_report(&report),
        ContactValidation::Email,
    );
}

#[test]
fn contact_validation_identifies_both_field_errors() {
    let input = CreateContactInput::from_untrusted("", "invalid");

    let report = match input.validate() {
        Ok(_) => panic!("expected invalid name and email"),
        Err(report) => report,
    };

    assert_eq!(
        ContactValidation::from_report(&report),
        ContactValidation::NameAndEmail,
    );
}
