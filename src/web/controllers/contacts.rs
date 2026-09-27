use sqlx::PgPool;

use crate::{
    application::contacts::list_contacts,
    domain::contact::ContactError,
    web::view_models::{ContactRowView, ContactsIndexView, ContactsListView},
};

pub(in crate::web) async fn index(
    pool: &PgPool,
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
) -> Result<ContactsIndexView, ContactError> {
    let contacts = contact_rows(pool).await?;

    Ok(ContactsIndexView::new(
        app_css_url,
        plain_css_url,
        bootstrap_js_url,
        htmx_js_url,
        contacts,
    ))
}

pub(in crate::web) async fn list(pool: &PgPool) -> Result<ContactsListView, ContactError> {
    Ok(ContactsListView::new(contact_rows(pool).await?))
}

async fn contact_rows(pool: &PgPool) -> Result<Vec<ContactRowView>, ContactError> {
    Ok(list_contacts(pool)
        .await?
        .into_iter()
        .map(|contact| ContactRowView::new(contact.id.to_string(), contact.name, contact.email))
        .collect())
}
