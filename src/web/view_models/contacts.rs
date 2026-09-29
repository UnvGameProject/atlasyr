use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub(in crate::web) struct ContactsPresentationAccess {
    can_create: bool,
    can_update: bool,
    can_delete: bool,
}

impl ContactsPresentationAccess {
    pub(in crate::web) fn new(can_create: bool, can_update: bool, can_delete: bool) -> Self {
        Self {
            can_create,
            can_update,
            can_delete,
        }
    }
}

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
    #[serde(flatten)]
    access: ContactsPresentationAccess,

    form: ContactCreateFormView,
    contacts: Vec<ContactRowView>,
}

impl ContactsWorkspaceView {
    pub(in crate::web) fn new(
        access: ContactsPresentationAccess,
        form: ContactCreateFormView,
        contacts: Vec<ContactRowView>,
    ) -> Self {
        Self {
            access,
            form,
            contacts,
        }
    }
}

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactsListView {
    #[serde(flatten)]
    access: ContactsPresentationAccess,

    contacts: Vec<ContactRowView>,
}

impl ContactsListView {
    pub(in crate::web) fn new(
        access: ContactsPresentationAccess,
        contacts: Vec<ContactRowView>,
    ) -> Self {
        Self { access, contacts }
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
    delete_open: bool,
    success_message: Option<&'static str>,
    validation_message: Option<&'static str>,
    error_message: Option<&'static str>,
    delete_error_message: Option<&'static str>,
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
            delete_open: false,
            success_message: None,
            validation_message: None,
            error_message: None,
            delete_error_message: None,
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
        self.delete_open = false;
        self.success_message = None;
        self.validation_message = validation_message;
        self.error_message = error_message;
        self.delete_error_message = None;
        self.name_error = name_error;
        self.email_error = email_error;
        self.unavailable_message = None;

        self
    }

    pub(in crate::web) fn updated(mut self, message: &'static str) -> Self {
        self.edit_name = self.name.clone();
        self.edit_email = self.email.clone();
        self.edit_open = false;
        self.delete_open = false;
        self.success_message = Some(message);
        self.validation_message = None;
        self.error_message = None;
        self.delete_error_message = None;
        self.name_error = None;
        self.email_error = None;
        self.unavailable_message = None;

        self
    }

    pub(in crate::web) fn delete_failed(mut self, message: &'static str) -> Self {
        self.edit_open = false;
        self.delete_open = true;
        self.success_message = None;
        self.validation_message = None;
        self.error_message = None;
        self.delete_error_message = Some(message);
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
            delete_open: false,
            success_message: None,
            validation_message: None,
            error_message: None,
            delete_error_message: None,
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
    #[serde(flatten)]
    access: ContactsPresentationAccess,

    row: ContactRowView,
}

impl ContactRowFragmentView {
    pub(in crate::web) fn new(access: ContactsPresentationAccess, row: ContactRowView) -> Self {
        Self { access, row }
    }
}

#[derive(Debug, Serialize)]
pub(in crate::web) struct ContactDeletedView {}

impl ContactDeletedView {
    pub(in crate::web) fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod presentation_tests {
    use std::path::Path;

    use minijinja::{Environment, UndefinedBehavior, path_loader};

    use super::{
        ContactCreateFormView, ContactRowView, ContactsPresentationAccess, ContactsWorkspaceView,
    };

    fn render_workspace(access: ContactsPresentationAccess) -> String {
        let mut environment = Environment::new();

        environment.set_loader(path_loader(Path::new("resources/views")));

        environment.set_undefined_behavior(UndefinedBehavior::Strict);

        let workspace = ContactsWorkspaceView::new(
            access,
            ContactCreateFormView::empty(),
            vec![ContactRowView::display(
                "00000000-0000-0000-0000-000000000001".to_owned(),
                "Ada Lovelace".to_owned(),
                "ada@example.com".to_owned(),
            )],
        );

        environment
            .get_template("fragments/contacts/workspace.html")
            .expect("Contacts workspace template should compile")
            .render(&workspace)
            .expect("Contacts workspace should render")
    }

    #[test]
    fn viewer_workspace_omits_mutation_controls() {
        let rendered = render_workspace(ContactsPresentationAccess::new(false, false, false));

        assert!(
            !rendered.contains("action=\"/contacts/create\""),
            "viewer received create form: {rendered}",
        );

        assert!(
            !rendered.contains("action=\"/contacts/update\""),
            "viewer received update form: {rendered}",
        );

        assert!(
            !rendered.contains("action=\"/contacts/delete\""),
            "viewer received delete form: {rendered}",
        );

        assert!(
            !rendered.contains("Edit contact"),
            "viewer received edit control: {rendered}",
        );

        assert!(
            !rendered.contains("Delete contact"),
            "viewer received delete control: {rendered}",
        );

        assert!(
            !rendered.contains("Actions"),
            "viewer received actions heading: {rendered}",
        );

        assert!(
            rendered.contains("Ada Lovelace"),
            "viewer lost readable contact content",
        );

        assert!(
            rendered.contains("Refresh contacts"),
            "viewer lost read/refresh UX",
        );
    }

    #[test]
    fn operator_workspace_renders_mutation_controls() {
        let rendered = render_workspace(ContactsPresentationAccess::new(true, true, true));

        assert!(
            rendered.contains("action=\"/contacts/create\""),
            "operator create form missing: {rendered}",
        );

        assert!(
            rendered.contains("action=\"/contacts/update\""),
            "operator update form missing: {rendered}",
        );

        assert!(
            rendered.contains("action=\"/contacts/delete\""),
            "operator delete form missing: {rendered}",
        );

        assert!(
            rendered.contains("Edit contact"),
            "operator edit control missing: {rendered}",
        );

        assert!(
            rendered.contains("Delete contact"),
            "operator delete control missing: {rendered}",
        );

        assert!(
            rendered.contains("Actions"),
            "operator actions heading missing: {rendered}",
        );
    }
}
