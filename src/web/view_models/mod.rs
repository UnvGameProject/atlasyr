mod contacts;
mod home;
mod view_dx;

pub(in crate::web) use contacts::{
    ContactCreateFormView, ContactRowView, ContactsIndexView, ContactsListView,
    ContactsWorkspaceView,
};
pub(in crate::web) use home::HomeView;
pub(in crate::web) use view_dx::ViewDxDemoView;
