// TODO(auth): Remove this temporary lint allowance once the auth subsystem
// is wired through the public login/session entrypoints.
mod model;
mod persistence;

pub(crate) use model::{User, UserCredentials};

#[cfg(test)]
mod tests;
