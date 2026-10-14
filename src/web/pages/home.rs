use topcoat::{
    Result,
    asset::asset_config,
    context::{Cx, app_context},
    router::page,
    view::View,
};

use crate::web::{
    ViewRenderer,
    assets::{APP_CSS, BOOTSTRAP_JS, HTMX_JS, PLAIN_CSS},
    controllers,
    views::HOME_INDEX,
};

#[page("/")]
async fn home(cx: &Cx) -> Result<impl View> {
    let assets = asset_config(cx);

    let model = controllers::home::index(
        assets.resolve(APP_CSS),
        assets.resolve(PLAIN_CSS),
        assets.resolve(BOOTSTRAP_JS),
        assets.resolve(HTMX_JS),
    );

    let renderer = app_context::<ViewRenderer>(cx);

    let rendered = renderer.render(HOME_INDEX, &model)?;

    Ok(rendered.into_view(cx))
}
