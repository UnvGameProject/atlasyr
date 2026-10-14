mod persistence;

#[cfg(test)]
mod tests;

pub(crate) use persistence::{ContactAuditAction, record_contact_event};
