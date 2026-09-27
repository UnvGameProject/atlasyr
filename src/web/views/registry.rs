use super::ViewName;

pub(super) const APP_LAYOUT: ViewName = ViewName::trusted("layouts/app.html");

pub(in crate::web) const HOME_INDEX: ViewName = ViewName::trusted("pages/home/index.html");

pub(super) const REGISTERED_VIEWS: &[ViewName] = &[APP_LAYOUT, HOME_INDEX];
