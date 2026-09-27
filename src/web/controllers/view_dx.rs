use crate::web::view_models::ViewDxDemoView;

pub(in crate::web) fn demo(is_htmx_request: bool) -> ViewDxDemoView {
    ViewDxDemoView::new(is_htmx_request)
}
