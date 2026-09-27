use sqlx::PgPool;

use crate::{
    application::contacts::list_contacts,
    domain::contact::ContactError,
    web::view_models::{ContactRowView, ContactsIndexView},
};

pub(in crate::web) async fn index(
    pool: &PgPool,
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
) -> Result<ContactsIndexView, ContactError> {
    let contacts = list_contacts(pool)
        .await?
        .into_iter()
        .map(|contact| ContactRowView::new(contact.id.to_string(), contact.name, contact.email))
        .collect();

    Ok(ContactsIndexView::new(
        app_css_url,
        plain_css_url,
        bootstrap_js_url,
        htmx_js_url,
        contacts,
    ))
}
