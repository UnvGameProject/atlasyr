use std::path::{Path, PathBuf};

use minijinja::{Environment, UndefinedBehavior, path_loader};
use minijinja_autoreload::AutoReloader;
use serde::Serialize;

use super::{RenderedHtml, ViewName};

const VIEW_ROOT: &str = "resources/views";

enum RendererBackend {
    Reloading(AutoReloader),
    Cached(Environment<'static>),
}

pub(crate) struct ViewRenderer {
    backend: RendererBackend,
}

impl ViewRenderer {
    pub(crate) fn new() -> Self {
        let root = PathBuf::from(VIEW_ROOT);

        let backend = if cfg!(debug_assertions) {
            let reload_root = root.clone();

            RendererBackend::Reloading(AutoReloader::new(move |notifier| {
                notifier.watch_path(&reload_root, true);

                Ok(build_environment(&reload_root))
            }))
        } else {
            RendererBackend::Cached(build_environment(&root))
        };

        Self { backend }
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
                eprintln!("view rendering failed for {}: {error:#}", name.as_str(),);

                Err(error.into())
            }
        }
    }
}

fn build_environment(root: &Path) -> Environment<'static> {
    let mut environment = Environment::new();

    environment.set_loader(path_loader(root));
    environment.set_undefined_behavior(UndefinedBehavior::Strict);

    environment
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

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::build_environment;

    #[derive(Serialize)]
    struct EscapingModel<'a> {
        value: &'a str,
    }

    #[test]
    fn html_templates_escape_model_values() {
        let environment = build_environment(std::path::Path::new("resources/views"));

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
        let environment = build_environment(std::path::Path::new("resources/views"));

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
}
