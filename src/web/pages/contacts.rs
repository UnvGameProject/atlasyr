use crate::{
    application::authorization::{
        AuthorizationActor, ContactsCreateGrant, ContactsDeleteGrant, ContactsReadGrant,
        ContactsUpdateGrant, load_authorization_actor,
    },
    auth::require_auth,
};

use serde::Deserialize;
use sqlx::PgPool;
use topcoat::{
    Result,
    asset::asset_config,
    context::{Cx, app_context},
    htmx::hx_request,
    router::{
        content::Form,
        error::{RouterErrorExt, internal_server_error, see_other},
        page,
    },
    view::View,
};

use crate::web::{
    ViewRenderer,
    assets::{APP_CSS, BOOTSTRAP_JS, HTMX_JS, PLAIN_CSS},
    controllers,
    view_models::{ContactDeletedView, ContactRowFragmentView, ContactsPresentationAccess},
    views::{CONTACTS_DELETED, CONTACTS_INDEX, CONTACTS_LIST, CONTACTS_ROW, CONTACTS_WORKSPACE},
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

    // Legacy presentation snapshots remain in the submitted form contract for
    // this slice but never enter DeleteContactCommand or internal-error
    // reporting. A later form cleanup may remove them independently.
    name: String,
    email: String,
}

/// Resolve request authentication into the current application authorization
/// actor.
///
/// No session/token representation crosses this boundary. If the authenticated
/// user disappeared between session resolution and account lookup, the request
/// no longer has a usable principal and fails as unauthorized.
async fn require_authorization_actor(cx: &Cx, pool: &PgPool) -> Result<AuthorizationActor> {
    let identity = require_auth(cx).await?;

    let actor = load_authorization_actor(pool, identity.user_id())
        .await
        .map_err(internal_server_error)?;

    Ok(actor.ok_or_unauthorized()?)
}

/// Project application authorization into rendering hints only.
///
/// These booleans improve UX but authorize nothing. Every operation remains
/// independently protected by the opaque application grants from POC 4C.
fn contacts_presentation_access(actor: AuthorizationActor) -> ContactsPresentationAccess {
    ContactsPresentationAccess::new(
        actor.contacts_create().is_some(),
        actor.contacts_update().is_some(),
        actor.contacts_delete().is_some(),
    )
}

#[page("/contacts/list")]
async fn contacts_list_html(cx: &Cx) -> Result<impl View> {
    let pool = app_context::<PgPool>(cx);

    let actor = require_authorization_actor(cx, pool).await?;
    let presentation_access = contacts_presentation_access(actor);
    let read_access: ContactsReadGrant = actor.contacts_read().ok_or_forbidden()?;

    let model = controllers::contacts::list(pool, read_access, presentation_access)
        .await
        .map_err(internal_server_error)?;

    let renderer = app_context::<ViewRenderer>(cx);

    let rendered = renderer.render(CONTACTS_LIST, &model)?;

    Ok(rendered.into_view(cx))
}

#[page("/contacts/{contact_id}/row")]
async fn contacts_row_html(cx: &Cx) -> Result<impl View> {
    let pool = app_context::<PgPool>(cx);

    let actor = require_authorization_actor(cx, pool).await?;
    let presentation_access = contacts_presentation_access(actor);
    let read_access: ContactsReadGrant = actor.contacts_read().ok_or_forbidden()?;

    let id: &str = topcoat::router::path_param::<ContactId>(cx);

    let row = controllers::contacts::row(pool, read_access, id)
        .await
        .map_err(internal_server_error)?;

    let renderer = app_context::<ViewRenderer>(cx);

    let model = ContactRowFragmentView::new(presentation_access, row);

    let rendered = renderer.render(CONTACTS_ROW, &model)?;

    Ok(rendered.into_view(cx))
}

#[page(POST "/contacts/create")]
async fn contacts_create_html(cx: &Cx, Form(input): Form<ContactCreateForm>) -> Result<impl View> {
    let pool = app_context::<PgPool>(cx);

    let actor = require_authorization_actor(cx, pool).await?;
    let presentation_access = contacts_presentation_access(actor);

    let create_access: ContactsCreateGrant = actor.contacts_create().ok_or_forbidden()?;

    let read_access: ContactsReadGrant = actor.contacts_read().ok_or_forbidden()?;

    let result = controllers::contacts::create(
        pool,
        create_access,
        read_access,
        presentation_access,
        input.name,
        input.email,
    )
    .await
    .map_err(internal_server_error)?;

    let renderer = app_context::<ViewRenderer>(cx);

    if hx_request(cx) {
        let rendered = renderer.render(CONTACTS_WORKSPACE, result.workspace())?;

        return Ok(rendered.into_view(cx));
    }

    if result.succeeded() {
        return Err(see_other("/contacts").into());
    }

    let workspace = result.into_workspace();

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
    let pool = app_context::<PgPool>(cx);

    let actor = require_authorization_actor(cx, pool).await?;
    let presentation_access = contacts_presentation_access(actor);

    let update_access: ContactsUpdateGrant = actor.contacts_update().ok_or_forbidden()?;

    let read_access: ContactsReadGrant = actor.contacts_read().ok_or_forbidden()?;

    let result = controllers::contacts::update(
        pool,
        update_access,
        read_access,
        input.id,
        input.name,
        input.email,
    )
    .await
    .map_err(internal_server_error)?;

    let renderer = app_context::<ViewRenderer>(cx);

    if hx_request(cx) {
        let model = ContactRowFragmentView::new(presentation_access, result.into_row());

        let rendered = renderer.render(CONTACTS_ROW, &model)?;

        return Ok(rendered.into_view(cx));
    }

    if result.succeeded() {
        return Err(see_other("/contacts").into());
    }

    let row = result.into_row();

    let workspace =
        controllers::contacts::workspace_with_row(pool, read_access, presentation_access, row)
            .await
            .map_err(internal_server_error)?;

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
    let pool = app_context::<PgPool>(cx);

    let actor = require_authorization_actor(cx, pool).await?;
    let presentation_access = contacts_presentation_access(actor);

    let delete_access: ContactsDeleteGrant = actor.contacts_delete().ok_or_forbidden()?;

    let read_access: ContactsReadGrant = actor.contacts_read().ok_or_forbidden()?;

    let result =
        controllers::contacts::delete(pool, delete_access, input.id, input.name, input.email)
            .await
            .map_err(internal_server_error)?;

    let renderer = app_context::<ViewRenderer>(cx);

    if hx_request(cx) {
        let rendered = match result {
            controllers::contacts::ContactDeleteResult::Deleted => {
                renderer.render(CONTACTS_DELETED, &ContactDeletedView::new())?
            }

            controllers::contacts::ContactDeleteResult::Row(row) => renderer.render(
                CONTACTS_ROW,
                &ContactRowFragmentView::new(presentation_access, row),
            )?,
        };

        return Ok(rendered.into_view(cx));
    }

    let row = match result {
        controllers::contacts::ContactDeleteResult::Deleted => {
            return Err(see_other("/contacts").into());
        }

        controllers::contacts::ContactDeleteResult::Row(row) => row,
    };

    let workspace =
        controllers::contacts::workspace_with_row(pool, read_access, presentation_access, row)
            .await
            .map_err(internal_server_error)?;

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
    let pool = app_context::<PgPool>(cx);

    let actor = require_authorization_actor(cx, pool).await?;
    let presentation_access = contacts_presentation_access(actor);
    let read_access: ContactsReadGrant = actor.contacts_read().ok_or_forbidden()?;

    let assets = asset_config(cx);

    let model = controllers::contacts::index(
        pool,
        read_access,
        presentation_access,
        assets.resolve(APP_CSS),
        assets.resolve(PLAIN_CSS),
        assets.resolve(BOOTSTRAP_JS),
        assets.resolve(HTMX_JS),
    )
    .await
    .map_err(internal_server_error)?;

    let renderer = app_context::<ViewRenderer>(cx);

    let rendered = renderer.render(CONTACTS_INDEX, &model)?;

    Ok(rendered.into_view(cx))
}
