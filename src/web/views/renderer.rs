use std::path::{Path, PathBuf};

use minijinja::{Environment, UndefinedBehavior, path_loader};
use minijinja_autoreload::AutoReloader;
use serde::Serialize;

use crate::config::AppEnvironment;

use super::{RenderedHtml, ViewName, registry::REGISTERED_VIEWS};

const VIEW_ROOT: &str = "resources/views";

enum RendererBackend {
    Reloading(AutoReloader),
    Cached(Environment<'static>),
}

pub(crate) struct ViewRenderer {
    app_environment: AppEnvironment,
    backend: RendererBackend,
}

impl ViewRenderer {
    pub(crate) fn new(app_environment: AppEnvironment) -> Result<Self, minijinja::Error> {
        let root = PathBuf::from(VIEW_ROOT);

        let backend = if app_environment.reload_views() {
            let reload_root = root.clone();
            let debug_enabled = app_environment.view_debug_enabled();

            RendererBackend::Reloading(AutoReloader::new(move |notifier| {
                notifier.watch_path(&reload_root, true);

                Ok(build_environment(&reload_root, debug_enabled))
            }))
        } else {
            let environment = build_environment(&root, app_environment.view_debug_enabled());

            if app_environment.validate_views_at_startup() {
                validate_views(&environment, REGISTERED_VIEWS, app_environment)?;
            }

            RendererBackend::Cached(environment)
        };

        Ok(Self {
            app_environment,
            backend,
        })
    }

    pub(in crate::web) fn render<T>(
        &self,
        name: ViewName,
        model: &T,
    ) -> topcoat::Result<RenderedHtml>
    where
        T: Serialize,
    {
        let rendered = match &self.backend {
            RendererBackend::Reloading(reloader) => {
                let environment = reloader.acquire_env()?;

                render_with_environment(&environment, name, model)
            }

            RendererBackend::Cached(environment) => {
                render_with_environment(environment, name, model)
            }
        };

        match rendered {
            Ok(html) => Ok(RenderedHtml::new(html)),

            Err(error) => {
                log_view_error("render", self.app_environment, name, &error);

                // Topcoat converts an unrecognized handler error into its
                // generic internal-server-error response. The MiniJinja
                // diagnostic remains server-side and is never rendered into
                // the customer response.
                Err(error.into())
            }
        }
    }
}

fn build_environment(root: &Path, debug_enabled: bool) -> Environment<'static> {
    let mut environment = Environment::new();

    environment.set_loader(path_loader(root));
    environment.set_undefined_behavior(UndefinedBehavior::Strict);
    environment.set_debug(debug_enabled);

    environment
}

fn validate_views(
    environment: &Environment<'_>,
    names: &[ViewName],
    app_environment: AppEnvironment,
) -> Result<(), minijinja::Error> {
    for &name in names {
        if let Err(error) = environment.get_template(name.as_str()) {
            log_view_error("startup-validation", app_environment, name, &error);

            return Err(error);
        }
    }

    Ok(())
}

fn render_with_environment<T>(
    environment: &Environment<'_>,
    name: ViewName,
    model: &T,
) -> Result<String, minijinja::Error>
where
    T: Serialize,
{
    environment.get_template(name.as_str())?.render(model)
}

fn log_view_error(
    phase: &str,
    app_environment: AppEnvironment,
    name: ViewName,
    error: &minijinja::Error,
) {
    eprintln!(
        "{}",
        format_view_error(phase, app_environment, name, error,),
    );
}

fn format_view_error(
    phase: &str,
    app_environment: AppEnvironment,
    name: ViewName,
    error: &minijinja::Error,
) -> String {
    let prefix = format!(
        "view {phase} failed: environment={} view={} kind={:?} line={:?}",
        app_environment.as_str(),
        name.as_str(),
        error.kind(),
        error.line(),
    );

    if app_environment.view_debug_enabled() {
        // Development and testing intentionally retain MiniJinja's rich
        // diagnostic formatting, including source context when available.
        format!("{prefix}: {error:#}")
    } else {
        // Production deliberately avoids alternate Error formatting.
        // MiniJinja parse errors can retain basic template source even when
        // Environment::set_debug(false) is configured, so production logging
        // must not render the Error's rich source context.
        match error.detail() {
            Some(detail) => {
                format!("{prefix} detail={detail}")
            }

            None => prefix,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use minijinja::ErrorKind;
    use serde::Serialize;
    use uuid::Uuid;

    use crate::config::AppEnvironment;

    use super::{
        REGISTERED_VIEWS, RendererBackend, ViewName, ViewRenderer, build_environment,
        validate_views,
    };

    #[derive(Serialize)]
    struct EscapingModel<'a> {
        value: &'a str,
    }

    #[test]
    fn html_templates_escape_model_values() {
        let environment = build_environment(Path::new("resources/views"), true);

        let template = environment
            .template_from_named_str("escaping-test.html", "{{ value }}")
            .expect("test template should compile");

        let rendered = template
            .render(EscapingModel {
                value: "<script>alert('xss')</script>",
            })
            .expect("test template should render");

        assert!(
            !rendered.contains("<script>"),
            "raw opening script tag escaped incorrectly: {rendered}",
        );

        assert!(
            !rendered.contains("</script>"),
            "raw closing script tag escaped incorrectly: {rendered}",
        );

        assert!(
            rendered.contains("&lt;script&gt;"),
            "escaped opening script tag was not present: {rendered}",
        );
    }

    #[test]
    fn undefined_template_values_fail_closed() {
        let environment = build_environment(Path::new("resources/views"), true);

        let template = environment
            .template_from_named_str("strict-test.html", "{{ missing_value }}")
            .expect("test template should compile");

        let error = template
            .render(())
            .expect_err("missing values must not render silently");

        assert!(
            error.to_string().contains("undefined"),
            "unexpected error: {error:#}",
        );
    }

    #[test]
    fn registered_views_validate_in_cached_mode() {
        let environment = build_environment(Path::new("resources/views"), false);

        validate_views(&environment, REGISTERED_VIEWS, AppEnvironment::Production)
            .expect("all registered application views should compile");
    }

    #[test]
    fn startup_validation_rejects_malformed_registered_view() {
        let root = std::env::temp_dir().join(format!("topcoat-poc-view-test-{}", Uuid::new_v4(),));

        fs::create_dir_all(&root).expect("temporary view root should be created");

        fs::write(root.join("broken.html"), "{% if %}")
            .expect("broken test template should be written");

        let environment = build_environment(&root, false);

        let names = [ViewName::trusted("broken.html")];

        let error = validate_views(&environment, &names, AppEnvironment::Production)
            .expect_err("malformed registered view must fail validation");

        fs::remove_dir_all(&root).expect("temporary view root should be removed");

        assert_eq!(error.kind(), ErrorKind::SyntaxError,);
    }

    #[test]
    fn development_uses_reloading_renderer() {
        let renderer = ViewRenderer::new(AppEnvironment::Development)
            .expect("development renderer should initialize");

        assert!(matches!(renderer.backend, RendererBackend::Reloading(_),));
    }

    #[test]
    fn testing_uses_cached_renderer() {
        let renderer = ViewRenderer::new(AppEnvironment::Testing)
            .expect("testing renderer should validate and initialize");

        assert!(matches!(renderer.backend, RendererBackend::Cached(_),));
    }

    #[test]
    fn production_uses_cached_renderer() {
        let renderer = ViewRenderer::new(AppEnvironment::Production)
            .expect("production renderer should validate and initialize");

        assert!(matches!(renderer.backend, RendererBackend::Cached(_),));
    }

    #[test]
    fn development_errors_retain_template_source() {
        let environment = build_environment(Path::new("resources/views"), true);

        assert!(environment.debug());

        let error = environment
            .template_from_named_str("development-error.html", "{% if %}")
            .expect_err("malformed template should fail");

        assert!(
            error.template_source().is_some(),
            "development diagnostics should retain template source",
        );
    }

    #[test]
    fn production_disables_rich_debug_mode() {
        let environment = build_environment(Path::new("resources/views"), false);

        assert!(
            !environment.debug(),
            "production must explicitly disable MiniJinja rich debug mode",
        );
    }

    #[test]
    fn production_log_output_does_not_expose_template_source() {
        const SOURCE_MARKER: &str = "SLICE2_PRODUCTION_TEMPLATE_SOURCE_SECRET";

        let environment = build_environment(Path::new("resources/views"), false);

        let error = environment
            .template_from_named_str(
                "production-broken.html",
                "{% if %}\nSLICE2_PRODUCTION_TEMPLATE_SOURCE_SECRET",
            )
            .expect_err("malformed production template should fail compilation");

        // MiniJinja 2.24 can attach basic template source to parse errors
        // independently of Environment::set_debug(false). Confirm this test
        // exercises that case before verifying our own output boundary.
        assert!(
            error
                .template_source()
                .is_some_and(|source| source.contains(SOURCE_MARKER)),
            "expected parse error to retain basic source context",
        );

        let output = super::format_view_error(
            "startup-validation",
            AppEnvironment::Production,
            ViewName::trusted("production-broken.html"),
            &error,
        );

        assert!(
            output.contains("environment=production"),
            "production log should identify its environment: {output}",
        );

        assert!(
            output.contains("view=production-broken.html"),
            "production log should identify the logical view: {output}",
        );

        assert!(
            !output.contains(SOURCE_MARKER),
            "production log leaked template source: {output}",
        );

        assert!(
            !output.contains("{% if %}"),
            "production log leaked template syntax: {output}",
        );
    }
}
