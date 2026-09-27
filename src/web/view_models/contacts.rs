use serde::Serialize;

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactsIndexView {
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,

    #[serde(flatten)]
    workspace: ContactsWorkspaceView,
}

impl ContactsIndexView {
    pub(in crate::web) fn new(
        app_css_url: String,
        plain_css_url: String,
        bootstrap_js_url: String,
        htmx_js_url: String,
        workspace: ContactsWorkspaceView,
    ) -> Self {
        Self {
            app_css_url,
            plain_css_url,
            bootstrap_js_url,
            htmx_js_url,
            workspace,
        }
    }
}

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactsWorkspaceView {
    form: ContactCreateFormView,
    contacts: Vec<ContactRowView>,
}

impl ContactsWorkspaceView {
    pub(in crate::web) fn new(form: ContactCreateFormView, contacts: Vec<ContactRowView>) -> Self {
        Self { form, contacts }
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
pub(in crate::web) struct ContactCreateFormView {
    name: String,
    email: String,
    success_message: Option<&'static str>,
    validation_message: Option<&'static str>,
    error_message: Option<&'static str>,
    name_error: Option<&'static str>,
    email_error: Option<&'static str>,
}

impl ContactCreateFormView {
    pub(in crate::web) fn new(
        name: String,
        email: String,
        success_message: Option<&'static str>,
        validation_message: Option<&'static str>,
        error_message: Option<&'static str>,
        name_error: Option<&'static str>,
        email_error: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            email,
            success_message,
            validation_message,
            error_message,
            name_error,
            email_error,
        }
    }

    pub(in crate::web) fn empty() -> Self {
        Self::new(String::new(), String::new(), None, None, None, None, None)
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
