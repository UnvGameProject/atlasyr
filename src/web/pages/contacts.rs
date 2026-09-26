use topcoat::{Result, context::Cx, router::page, runtime::signal, view::View};

use self::{index::contacts_index, view_model::ContactsIndexViewModel};

mod create;
mod create_form;
mod delete;
mod index;
mod list;
mod row;
#[cfg(test)]
mod tests;
mod update;
mod validation;
mod view_model;

#[page("/contacts")]
async fn contacts(cx: &Cx) -> Result<impl View> {
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
