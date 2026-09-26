// TODO(auth): Remove this temporary lint allowance once the auth subsystem
// is wired through the public login/session entrypoints.
#![allow(dead_code)]

mod model;
mod persistence;

#[cfg(test)]
mod tests;

pub(crate) use model::UserSession;
