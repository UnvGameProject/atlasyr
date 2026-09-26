mod authenticate;

#[cfg(test)]
mod tests;

pub(crate) use authenticate::{
    AuthenticateUserCommand, AuthenticateUserOutcome, authenticate_user,
};
