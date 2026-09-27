use serde::Serialize;

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactsIndexView {
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
    contacts: Vec<ContactRowView>,
}

impl ContactsIndexView {
    pub(in crate::web) fn new(
        app_css_url: String,
        plain_css_url: String,
        bootstrap_js_url: String,
        htmx_js_url: String,
        contacts: Vec<ContactRowView>,
    ) -> Self {
        Self {
            app_css_url,
            plain_css_url,
            bootstrap_js_url,
            htmx_js_url,
            contacts,
        }
    }
}

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactsListView {
    contacts: Vec<ContactRowView>,
}

impl ContactsListView {
    pub(in crate::web) fn new(contacts: Vec<ContactRowView>) -> Self {
        Self { contacts }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(in crate::web) struct ContactRowView {
    id: String,
    name: String,
    email: String,
}

impl ContactRowView {
    pub(in crate::web) fn new(id: String, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}
