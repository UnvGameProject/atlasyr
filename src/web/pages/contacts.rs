use crate::auth::require_auth;

use sqlx::PgPool;
use topcoat::{
    Result,
    asset::asset_config,
    context::{Cx, app_context},
    router::page,
    runtime::signal,
    view::View,
};

use crate::web::{
    ViewRenderer,
    assets::{APP_CSS, BOOTSTRAP_JS, HTMX_JS, PLAIN_CSS},
    controllers,
    views::{
        CONTACTS_INDEX,
        contacts::{ContactsIndexViewModel, contacts_index},
    },
};

#[page("/(native)/contacts")]
async fn contacts(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;

    let view_model = ContactsIndexViewModel {
        name: signal(cx, String::new),
        email: signal(cx, String::new),
        status: signal(cx, String::new),
        name_error: signal(cx, String::new),
        email_error: signal(cx, String::new),
        refresh: signal(cx, || 0.0),
    };

    contacts_index(cx, view_model).await
}

#[page("/view-dx/contacts")]
async fn contacts_html(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);
    let assets = asset_config(cx);

    let model = controllers::contacts::index(
        pool,
        assets.resolve(APP_CSS),
        assets.resolve(PLAIN_CSS),
        assets.resolve(BOOTSTRAP_JS),
        assets.resolve(HTMX_JS),
    )
    .await?;

    let renderer = app_context::<ViewRenderer>(cx);

    let rendered = renderer.render(CONTACTS_INDEX, &model)?;

    Ok(rendered.into_view(cx))
}
