use crate::web::view_models::HomeView;

pub(in crate::web) fn index(
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
    htmx_js_url: String,
) -> HomeView {
    HomeView::new(app_css_url, plain_css_url, bootstrap_js_url, htmx_js_url)
}
