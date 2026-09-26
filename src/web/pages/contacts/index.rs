use topcoat::{
    Result,
    context::Cx,
    view::{View, view},
};

use super::{
    create_form::contact_create_form, list::contact_list, view_model::ContactsIndexViewModel,
};

pub(super) async fn contacts_index(
    cx: &Cx,
    view_model: ContactsIndexViewModel,
) -> Result<impl View> {
    let ContactsIndexViewModel {
        name,
        email,
        status,
        name_error,
        email_error,
        refresh,
    } = view_model;

    Ok(view! { cx =>
        <main class="container py-5">
            <div class="d-flex justify-content-between align-items-center mb-4">
                <div>
                    <h1 class="mb-1">"Contacts"</h1>
                    <p class="text-body-secondary mb-0">
                        "Topcoat + SQLx contact workflow"
                    </p>
                </div>

                <a class="btn btn-outline-secondary" href="/">
                    "Home"
                </a>
            </div>

            <div class="row g-4">
                contact_create_form(
                    name: &name,
                    email: &email,
                    status: &status,
                    name_error: &name_error,
                    email_error: &email_error,
                    refresh: &refresh,
                )

                <section class="col-lg-7">
                    <div class="d-flex align-items-center justify-content-between mb-3">
                        <h2 class="h4 mb-0">"Existing contacts"</h2>
                    </div>

                    contact_list(refresh: $(refresh))
                </section>
            </div>
        </main>
    })
}
