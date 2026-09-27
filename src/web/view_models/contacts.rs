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
    edit_name: String,
    edit_email: String,
    edit_open: bool,
    success_message: Option<&'static str>,
    validation_message: Option<&'static str>,
    error_message: Option<&'static str>,
    name_error: Option<&'static str>,
    email_error: Option<&'static str>,
    unavailable_message: Option<&'static str>,
}

impl ContactRowView {
    pub(in crate::web) fn display(id: String, name: String, email: String) -> Self {
        Self {
            id,
            edit_name: name.clone(),
            edit_email: email.clone(),
            name,
            email,
            edit_open: false,
            success_message: None,
            validation_message: None,
            error_message: None,
            name_error: None,
            email_error: None,
            unavailable_message: None,
        }
    }

    pub(in crate::web) fn editing(
        mut self,
        edit_name: String,
        edit_email: String,
        validation_message: Option<&'static str>,
        error_message: Option<&'static str>,
        name_error: Option<&'static str>,
        email_error: Option<&'static str>,
    ) -> Self {
        self.edit_name = edit_name;
        self.edit_email = edit_email;
        self.edit_open = true;
        self.success_message = None;
        self.validation_message = validation_message;
        self.error_message = error_message;
        self.name_error = name_error;
        self.email_error = email_error;
        self.unavailable_message = None;

        self
    }

    pub(in crate::web) fn updated(mut self, message: &'static str) -> Self {
        self.edit_name = self.name.clone();
        self.edit_email = self.email.clone();
        self.edit_open = false;
        self.success_message = Some(message);
        self.validation_message = None;
        self.error_message = None;
        self.name_error = None;
        self.email_error = None;
        self.unavailable_message = None;

        self
    }

    pub(in crate::web) fn unavailable(id: String, message: &'static str) -> Self {
        Self {
            id,
            name: String::new(),
            email: String::new(),
            edit_name: String::new(),
            edit_email: String::new(),
            edit_open: false,
            success_message: None,
            validation_message: None,
            error_message: None,
            name_error: None,
            email_error: None,
            unavailable_message: Some(message),
        }
    }

    pub(in crate::web) fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactRowFragmentView {
    row: ContactRowView,
}

impl ContactRowFragmentView {
    pub(in crate::web) fn new(row: ContactRowView) -> Self {
        Self { row }
    }
}
