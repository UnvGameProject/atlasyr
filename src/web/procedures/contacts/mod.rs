mod create;
mod delete;
#[cfg(test)]
mod tests;
mod update;
mod validation;

pub(in crate::web) use create::create_contact;
pub(in crate::web) use delete::delete_contact;
pub(in crate::web) use update::update_contact;
