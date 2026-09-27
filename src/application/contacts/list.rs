use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::models::contact::Contact, domain::contact::ContactError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContactListItem {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) email: String,
}

impl From<Contact> for ContactListItem {
    fn from(contact: Contact) -> Self {
        Self {
            id: contact.id,
            name: contact.name,
            email: contact.email,
        }
    }
}

pub(crate) async fn find_contact(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ContactListItem>, ContactError> {
    Contact::find_by_id(pool, id)
        .await
        .map(|contact| contact.map(ContactListItem::from))
        .map_err(ContactError::storage)
}

pub(crate) async fn list_contacts(pool: &PgPool) -> Result<Vec<ContactListItem>, ContactError> {
    Contact::find_all(pool)
        .await
        .map(|contacts| contacts.into_iter().map(ContactListItem::from).collect())
        .map_err(ContactError::storage)
}
