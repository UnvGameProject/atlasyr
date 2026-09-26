// TODO(auth): Remove this temporary lint allowance once the auth subsystem
// is wired through the public login/session entrypoints.
#![allow(dead_code)]

mod create;
mod error;

pub(crate) use create::CreateUserInput;
pub(crate) use error::UserError;

#[cfg(test)]
mod tests;
