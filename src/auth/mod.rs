// TODO(auth): Remove this temporary lint allowance once the auth subsystem
// is wired through the public login/session entrypoints.
#![allow(dead_code)]

mod driver;
mod manager;

pub(crate) mod password;
pub(crate) mod session;

pub(crate) use manager::{AuthError, AuthManager, CredentialOutcome};
