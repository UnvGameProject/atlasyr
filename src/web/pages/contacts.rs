use crate::auth::require_auth;

use topcoat::{Result, context::Cx, router::page, runtime::signal, view::View};

use crate::web::views::contacts::{ContactsIndexViewModel, contacts_index};

#[page("/contacts")]
async fn contacts(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;

    let view_model = ContactsIndexViewModel {
        name: signal(cx, String::new),
        email: signal(cx, String::new),
        status: signal(cx, String::new),
        name_error: signal(cx, String::new),
        email_error: signal(cx, String::new),
        refresh: signal(cx, || 0.0),
    };

    contacts_index(cx, view_model).await
}
