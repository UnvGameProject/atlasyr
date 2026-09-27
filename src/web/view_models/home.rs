use serde::Serialize;

#[derive(Debug, Serialize)]
pub(in crate::web) struct HomeView {
    app_css_url: String,
    plain_css_url: String,
    bootstrap_js_url: String,
}

impl HomeView {
    pub(in crate::web) fn new(
        app_css_url: String,
        plain_css_url: String,
        bootstrap_js_url: String,
    ) -> Self {
        Self {
            app_css_url,
            plain_css_url,
            bootstrap_js_url,
        }
    }
}
