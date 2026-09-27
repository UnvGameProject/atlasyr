use super::ViewName;

pub(super) const APP_LAYOUT: ViewName = ViewName::trusted("layouts/app.html");

pub(in crate::web) const HOME_INDEX: ViewName = ViewName::trusted("pages/home/index.html");

pub(in crate::web) const VIEW_DX_DEMO: ViewName = ViewName::trusted("fragments/view_dx/demo.html");

pub(in crate::web) const CONTACTS_INDEX: ViewName = ViewName::trusted("pages/contacts/index.html");

pub(in crate::web) const CONTACTS_LIST: ViewName =
    ViewName::trusted("fragments/contacts/list.html");

pub(super) const REGISTERED_VIEWS: &[ViewName] = &[
    APP_LAYOUT,
    HOME_INDEX,
    VIEW_DX_DEMO,
    CONTACTS_INDEX,
    CONTACTS_LIST,
];
