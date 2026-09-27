use topcoat::{
    context::Cx,
    view::{Unescaped, View, view},
};

pub(in crate::web) struct RenderedHtml {
    html: String,
}

impl RenderedHtml {
    pub(super) fn new(html: String) -> Self {
        Self { html }
    }

    pub(in crate::web) fn into_view(self, cx: &Cx) -> impl View {
        // SECURITY BOUNDARY:
        //
        // MiniJinja has already escaped ordinary model values before the
        // rendered document reaches this point. This module is the single
        // application-owned bridge permitted to mark complete renderer output
        // as trusted Topcoat HTML.
        //
        // This is a plain Rust function rather than a Topcoat #[page],
        // #[layout], #[component], or #[shard], so the request context must be
        // supplied explicitly to view!.
        view! { cx =>
            (Unescaped::new_unchecked(self.html))
        }
    }
}
