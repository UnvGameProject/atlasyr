// TODO(auth): Remove these temporary lint allowances once the auth subsystem
// is wired through the public login/session entrypoints.
#![allow(dead_code, unused_imports)]

mod authenticate;

#[cfg(test)]
mod tests;

pub(crate) use authenticate::{
    AuthenticateUserCommand, AuthenticateUserError, AuthenticateUserOutcome, authenticate_user,
};
