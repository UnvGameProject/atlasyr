use topcoat::runtime::Signal;

/// Reactive state required to render and operate the Contacts index page.
///
/// Signals are created by the page/controller and passed into the view rather
/// than created by the presentation module. This keeps Topcoat runtime
/// ownership visible at the page boundary.
pub(super) struct ContactsIndexViewModel {
    pub(super) name: Signal<String>,
    pub(super) email: Signal<String>,
    pub(super) status: Signal<String>,
    pub(super) name_error: Signal<String>,
    pub(super) email_error: Signal<String>,
    pub(super) refresh: Signal<f64>,
}
