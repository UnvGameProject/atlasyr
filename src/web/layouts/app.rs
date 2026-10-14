use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{Slot, layout},
    runtime::{Event, signal},
    view::{View, view},
};

use crate::{
    application::auth::{CurrentUser, load_current_user},
    auth::current_identity,
    web::{
        assets::{APP_CSS, BOOTSTRAP_JS, PLAIN_CSS},
        procedures::auth::logout,
    },
};

#[layout("/(native)")]
async fn app_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let pool = app_context::<PgPool>(cx);

    let current_user: Option<CurrentUser> = match current_identity(cx).await? {
        Some(identity) => load_current_user(pool, identity.user_id()).await?,

        None => None,
    };

    let logout_status = signal(cx, String::new);

    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" data-bs-theme="dark">
            <head>
                <meta charset="utf-8">
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1"
                >
                <meta
                    name="color-scheme"
                    content="dark light"
                >

                <title>"Atlasyr"</title>

                <link
                    rel="stylesheet"
                    href=(APP_CSS)
                >
                <link
                    rel="stylesheet"
                    href=(PLAIN_CSS)
                >

                <script
                    src=(BOOTSTRAP_JS)
                    defer=""
                ></script>

                topcoat::runtime::script()
                topcoat::dev::script()
            </head>

            <body>
                <nav
                    class="navbar navbar-expand border-bottom bg-body-tertiary"
                    aria-label="Primary navigation"
                >
                    <div class="container">
                        <a
                            class="navbar-brand fw-semibold"
                            href="/"
                        >
                            "Atlasyr"
                        </a>

                        <div class="d-flex align-items-center gap-3">
                            <a
                                class="nav-link"
                                href="/"
                            >
                                "Home"
                            </a>

                            match current_user {
                                Some(user) => {
                                    <a
                                        class="nav-link"
                                        href="/contacts"
                                    >
                                        "Contacts"
                                    </a>

                                    <span
                                        class="small text-body-secondary"
                                    >
                                        (user.email())
                                    </span>

                                    <button
                                        class="btn btn-outline-secondary btn-sm"
                                        type="button"
                                        :disabled=$(
                                            logout_status.get()
                                                == "submitting"
                                        )
                                        @click=$(
                                            async |_e: Event| {
                                                logout_status.set(
                                                    "submitting".to_owned()
                                                );

                                                let outcome =
                                                    logout().await;

                                                if outcome == "signed_out" {
                                                    raw!(
                                                        "window.location.assign('/login')"
                                                    );
                                                }

                                                if outcome == "error" {
                                                    logout_status.set(
                                                        "error".to_owned()
                                                    );
                                                }
                                            }
                                        )
                                    >
                                        "Sign out"
                                    </button>
                                },

                                None => {
                                    <a
                                        class="btn btn-primary btn-sm"
                                        href="/login"
                                    >
                                        "Sign in"
                                    </a>
                                },
                            }
                        </div>
                    </div>
                </nav>

                <div
                    class="container mt-3"
                    :hidden=$(
                        logout_status.get() != "error"
                    )
                >
                    <div
                        class="alert alert-danger mb-0"
                        role="alert"
                    >
                        "Sign-out could not be completed. Please try again."
                    </div>
                </div>

                (slot)
            </body>
        </html>
    })
}
