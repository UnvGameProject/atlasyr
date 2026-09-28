mod contacts;
mod home;
mod view_dx;

// RUSTY_VIEW_MODEL_MODULES

pub(in crate::web) use contacts::{
    ContactCreateFormView, ContactDeletedView, ContactRowFragmentView, ContactRowView,
    ContactsIndexView, ContactsListView, ContactsWorkspaceView,
};
pub(in crate::web) use home::HomeView;
pub(in crate::web) use view_dx::ViewDxDemoView;

// RUSTY_VIEW_MODEL_EXPORTS
