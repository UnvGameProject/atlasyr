use topcoat::{
    Result,
    context::Cx,
    router::{error::redirect, page},
    runtime::{Event, signal},
    view::{View, view},
};

use crate::{auth::current_identity, web::procedures::auth::login};

#[page("/login")]
async fn login_page(cx: &Cx) -> Result<impl View> {
    if current_identity(cx).await?.is_some() {
        return Err(redirect("/contacts").into());
    }

    let email = signal(cx, String::new);
    let password = signal(cx, String::new);
    let status = signal(cx, String::new);

    Ok(view! { cx =>
        <main class="container py-5">
            <div class="row justify-content-center py-5">
                <div class="col-12 col-md-8 col-lg-5">
                    <div class="card shadow-sm">
                        <div class="card-body p-4 p-md-5">
                            <div class="mb-4">
                                <h1 class="h3 mb-2">
                                    "Sign in"
                                </h1>

                                <p class="text-body-secondary mb-0">
                                    "Use your account credentials to continue."
                                </p>
                            </div>

                            <div
                                class="alert alert-info"
                                role="status"
                                :hidden=$(status.get() != "submitting")
                            >
                                "Signing in..."
                            </div>

                            <div
                                class="alert alert-danger"
                                role="alert"
                                :hidden=$(status.get() != "invalid_credentials")
                            >
                                "The email or password is incorrect."
                            </div>

                            <div
                                class="alert alert-warning"
                                role="alert"
                                :hidden=$(status.get() != "throttled")
                            >
                                "Too many sign-in attempts. Please try again shortly."
                            </div>

                            <div
                                class="alert alert-danger"
                                role="alert"
                                :hidden=$(status.get() != "error")
                            >
                                "Sign-in could not be completed. Please try again."
                            </div>

                            <form
                                @submit=$(async |e: Event| {
                                    e.prevent_default();

                                    status.set(
                                        "submitting".to_owned()
                                    );

                                    let outcome = login(
                                        email.get(),
                                        password.get(),
                                    )
                                    .await;

                                    if outcome == "authenticated" {
                                        raw!(
                                            "window.location.assign('/contacts')"
                                        );
                                    }

                                    if outcome == "invalid_credentials" {
                                        password.set(
                                            "".to_owned()
                                        );

                                        status.set(
                                            "invalid_credentials".to_owned()
                                        );
                                    }

                                    if outcome == "throttled" {
                                        password.set(
                                            "".to_owned()
                                        );

                                        status.set(
                                            "throttled".to_owned()
                                        );
                                    }

                                    if outcome == "error" {
                                        password.set(
                                            "".to_owned()
                                        );

                                        status.set(
                                            "error".to_owned()
                                        );
                                    }
                                })
                            >
                                <div class="mb-3">
                                    <label
                                        class="form-label"
                                        for="login-email"
                                    >
                                        "Email"
                                    </label>

                                    <input
                                        id="login-email"
                                        class="form-control"
                                        type="email"
                                        name="email"
                                        autocomplete="username"
                                        required=""
                                        :value=$(email.get())
                                        @input=$(|e: Event| {
                                            email.set(
                                                e.target.value
                                            );
                                        })
                                    >
                                </div>

                                <div class="mb-4">
                                    <label
                                        class="form-label"
                                        for="login-password"
                                    >
                                        "Password"
                                    </label>

                                    <input
                                        id="login-password"
                                        class="form-control"
                                        type="password"
                                        name="password"
                                        autocomplete="current-password"
                                        required=""
                                        :value=$(password.get())
                                        @input=$(|e: Event| {
                                            password.set(
                                                e.target.value
                                            );
                                        })
                                    >
                                </div>

                                <button
                                    class="btn btn-primary w-100"
                                    type="submit"
                                    :disabled=$(status.get() == "submitting")
                                >
                                    "Sign in"
                                </button>
                            </form>

                            <div class="text-center mt-4">
                                <a
                                    class="link-secondary"
                                    href="/"
                                >
                                    "Back to home"
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </main>
    })
}
