use topcoat::runtime::Signal;

/// Reactive state required to render and operate the Contacts index page.
///
/// Signals are created by the page/controller and passed into the view rather
/// than created by the presentation module. This keeps Topcoat runtime
/// ownership visible at the page boundary.
pub(in crate::web) struct ContactsIndexViewModel {
    pub(in crate::web) name: Signal<String>,
    pub(in crate::web) email: Signal<String>,
    pub(in crate::web) status: Signal<String>,
    pub(in crate::web) name_error: Signal<String>,
    pub(in crate::web) email_error: Signal<String>,
    pub(in crate::web) refresh: Signal<f64>,
}
