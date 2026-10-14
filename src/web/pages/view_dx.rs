use topcoat::{
    Result,
    context::{Cx, app_context},
    htmx::hx_request,
    router::page,
    view::View,
};

use crate::web::{ViewRenderer, controllers, views::VIEW_DX_DEMO};

#[page("/view-dx/demo")]
async fn view_dx_demo(cx: &Cx) -> Result<impl View> {
    // HX-Request is presentation metadata only.
    // It is not authentication or authorization.
    let model = controllers::view_dx::demo(hx_request(cx));

    let renderer = app_context::<ViewRenderer>(cx);

    let rendered = renderer.render(VIEW_DX_DEMO, &model)?;

    Ok(rendered.into_view(cx))
}
