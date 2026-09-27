use crate::auth::require_auth;

use serde::Deserialize;
use sqlx::PgPool;
use topcoat::{
    Result,
    asset::asset_config,
    context::{Cx, app_context},
    htmx::hx_request,
    router::{content::Form, page},
    runtime::signal,
    view::View,
};

use crate::web::{
    ViewRenderer,
    assets::{APP_CSS, BOOTSTRAP_JS, HTMX_JS, PLAIN_CSS},
    controllers,
    view_models::{ContactDeletedView, ContactRowFragmentView},
    views::{
        CONTACTS_DELETED, CONTACTS_INDEX, CONTACTS_LIST, CONTACTS_ROW, CONTACTS_WORKSPACE,
        contacts::{ContactsIndexViewModel, contacts_index},
    },
};

topcoat::router::path_param!(contact_id);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactCreateForm {
    name: String,
    email: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactUpdateForm {
    id: String,
    name: String,
    email: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactDeleteForm {
    id: String,

    // Presentation-only snapshots used only to render a generic
    // persistence failure. DeleteContactCommand remains ID-only.
    name: String,
    email: String,
}

#[page("/(native)/native/contacts")]
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

#[page("/contacts/list")]
async fn contacts_list_html(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);
    let model = controllers::contacts::list(pool).await?;

    let renderer = app_context::<ViewRenderer>(cx);

    let rendered = renderer.render(CONTACTS_LIST, &model)?;

    Ok(rendered.into_view(cx))
}

#[page("/contacts/{contact_id}/row")]
async fn contacts_row_html(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;

    let id: &str = topcoat::router::path_param::<ContactId>(cx);

    let pool = app_context::<PgPool>(cx);

    let row = controllers::contacts::row(pool, id).await?;

    let renderer = app_context::<ViewRenderer>(cx);

    let model = ContactRowFragmentView::new(row);

    let rendered = renderer.render(CONTACTS_ROW, &model)?;

    Ok(rendered.into_view(cx))
}

#[page(POST "/contacts/create")]
async fn contacts_create_html(cx: &Cx, Form(input): Form<ContactCreateForm>) -> Result<impl View> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);

    let workspace = controllers::contacts::create(pool, input.name, input.email).await?;

    let renderer = app_context::<ViewRenderer>(cx);

    if hx_request(cx) {
        let rendered = renderer.render(CONTACTS_WORKSPACE, &workspace)?;

        return Ok(rendered.into_view(cx));
    }

    let assets = asset_config(cx);

    let model = controllers::contacts::index_with_workspace(
        assets.resolve(APP_CSS),
        assets.resolve(PLAIN_CSS),
        assets.resolve(BOOTSTRAP_JS),
        assets.resolve(HTMX_JS),
        workspace,
    );

    let rendered = renderer.render(CONTACTS_INDEX, &model)?;

    Ok(rendered.into_view(cx))
}

#[page(POST "/contacts/update")]
async fn contacts_update_html(cx: &Cx, Form(input): Form<ContactUpdateForm>) -> Result<impl View> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);

    let row = controllers::contacts::update(pool, input.id, input.name, input.email).await?;

    let renderer = app_context::<ViewRenderer>(cx);

    if hx_request(cx) {
        let model = ContactRowFragmentView::new(row);

        let rendered = renderer.render(CONTACTS_ROW, &model)?;

        return Ok(rendered.into_view(cx));
    }

    let workspace = controllers::contacts::workspace_with_row(pool, row).await?;

    let assets = asset_config(cx);

    let model = controllers::contacts::index_with_workspace(
        assets.resolve(APP_CSS),
        assets.resolve(PLAIN_CSS),
        assets.resolve(BOOTSTRAP_JS),
        assets.resolve(HTMX_JS),
        workspace,
    );

    let rendered = renderer.render(CONTACTS_INDEX, &model)?;

    Ok(rendered.into_view(cx))
}

#[page(POST "/contacts/delete")]
async fn contacts_delete_html(cx: &Cx, Form(input): Form<ContactDeleteForm>) -> Result<impl View> {
    require_auth(cx).await?;

    let pool = app_context::<PgPool>(cx);

    let result = controllers::contacts::delete(pool, input.id, input.name, input.email).await?;

    let renderer = app_context::<ViewRenderer>(cx);

    if hx_request(cx) {
        let rendered = match result {
            controllers::contacts::ContactDeleteResult::Deleted => {
                renderer.render(CONTACTS_DELETED, &ContactDeletedView::new())?
            }

            controllers::contacts::ContactDeleteResult::Row(row) => {
                renderer.render(CONTACTS_ROW, &ContactRowFragmentView::new(row))?
            }
        };

        return Ok(rendered.into_view(cx));
    }

    let workspace = match result {
        controllers::contacts::ContactDeleteResult::Deleted => {
            controllers::contacts::workspace_default(pool).await?
        }

        controllers::contacts::ContactDeleteResult::Row(row) => {
            controllers::contacts::workspace_with_row(pool, row).await?
        }
    };

    let assets = asset_config(cx);

    let model = controllers::contacts::index_with_workspace(
        assets.resolve(APP_CSS),
        assets.resolve(PLAIN_CSS),
        assets.resolve(BOOTSTRAP_JS),
        assets.resolve(HTMX_JS),
        workspace,
    );

    let rendered = renderer.render(CONTACTS_INDEX, &model)?;

    Ok(rendered.into_view(cx))
}

#[page("/contacts")]
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
