#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContactValidation {
    Name,
    Email,
    NameAndEmail,
    General,
}

impl ContactValidation {
    pub(super) fn from_report(report: &garde::Report) -> Self {
        let name_invalid = garde::select!(report, name).next().is_some();
        let email_invalid = garde::select!(report, email).next().is_some();

        match (name_invalid, email_invalid) {
            (true, true) => Self::NameAndEmail,
            (true, false) => Self::Name,
            (false, true) => Self::Email,
            (false, false) => Self::General,
        }
    }
}
