mod actor;
mod policy;

#[cfg(test)]
mod tests;

pub(crate) use actor::{
    AuthorizationActor, ContactsCreateGrant, ContactsDeleteGrant, ContactsReadGrant,
    ContactsUpdateGrant, load_authorization_actor,
};
