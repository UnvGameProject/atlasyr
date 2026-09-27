mod contacts;
mod home;
mod view_dx;

pub(in crate::web) use contacts::{
    ContactCreateFormView, ContactDeletedView, ContactRowFragmentView, ContactRowView,
    ContactsIndexView, ContactsListView, ContactsWorkspaceView,
};
pub(in crate::web) use home::HomeView;
pub(in crate::web) use view_dx::ViewDxDemoView;
