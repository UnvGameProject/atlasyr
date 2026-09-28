pub(crate) mod auth;

// POC 4B proves the authorization policy in isolation before transport/use-case
// wiring. Remove this local allowance when the next slice consumes the module.
#[allow(dead_code)]
pub(crate) mod authorization;
pub(crate) mod contacts;
