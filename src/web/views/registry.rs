use super::ViewName;

pub(super) const APP_LAYOUT: ViewName = ViewName::trusted("layouts/app.html");

pub(in crate::web) const HOME_INDEX: ViewName = ViewName::trusted("pages/home/index.html");

pub(in crate::web) const VIEW_DX_DEMO: ViewName = ViewName::trusted("fragments/view_dx/demo.html");

pub(in crate::web) const CONTACTS_INDEX: ViewName = ViewName::trusted("pages/contacts/index.html");

pub(super) const CONTACTS_CREATE_FORM: ViewName =
    ViewName::trusted("fragments/contacts/create_form.html");

pub(in crate::web) const CONTACTS_WORKSPACE: ViewName =
    ViewName::trusted("fragments/contacts/workspace.html");

pub(in crate::web) const CONTACTS_LIST: ViewName =
    ViewName::trusted("fragments/contacts/list.html");

pub(in crate::web) const CONTACTS_ROW: ViewName = ViewName::trusted("fragments/contacts/row.html");

pub(in crate::web) const CONTACTS_DELETED: ViewName =
    ViewName::trusted("fragments/contacts/deleted.html");

// RUSTY_VIEW_CONSTANTS

pub(super) const REGISTERED_VIEWS: &[ViewName] = &[
    APP_LAYOUT,
    HOME_INDEX,
    VIEW_DX_DEMO,
    CONTACTS_INDEX,
    CONTACTS_CREATE_FORM,
    CONTACTS_WORKSPACE,
    CONTACTS_LIST,
    CONTACTS_ROW,
    CONTACTS_DELETED,
    // RUSTY_REGISTERED_VIEWS
];
