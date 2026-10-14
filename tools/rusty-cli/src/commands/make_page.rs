use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use syn::{Item, LitStr};

use crate::{cli::PageArgs, commands::CommandResult, project::Project};

const PAGE_MODULE_MARKER: &str = "// RUSTY_PAGE_MODULES";

const CONTROLLER_MODULE_MARKER: &str = "// RUSTY_CONTROLLER_MODULES";

const VIEW_MODEL_MODULE_MARKER: &str = "// RUSTY_VIEW_MODEL_MODULES";

const VIEW_MODEL_EXPORT_MARKER: &str = "// RUSTY_VIEW_MODEL_EXPORTS";

const VIEW_CONSTANT_MARKER: &str = "// RUSTY_VIEW_CONSTANTS";

const REGISTERED_VIEW_MARKER: &str = "    // RUSTY_REGISTERED_VIEWS";

const VIEW_EXPORT_MARKER: &str = "    // RUSTY_VIEW_EXPORTS";

pub(crate) fn run(args: PageArgs) -> CommandResult {
    let project = Project::discover()?;

    let names = PageNames::parse(&args.name)?;

    let route = RoutePath::parse(&args.path)?;

    scaffold_page(project.root(), &names, &route, args.dry_run)?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PageNames {
    module: String,
    type_name: String,
    constant: String,
    title: String,
}

impl PageNames {
    fn parse(raw: &str) -> io::Result<Self> {
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "page name cannot be empty",
            ));
        }

        if raw.contains('/') || raw.contains('\\') || raw.contains('.') {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "page name must be a single module name, not a path",
            ));
        }

        if !raw.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                || character.is_ascii_whitespace()
        }) {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "page name contains unsupported characters",
            ));
        }

        let module = raw.to_case(Case::Snake);

        validate_identifier(&module, "generated page module")?;

        let base_type = raw.to_case(Case::Pascal);

        if base_type.is_empty()
            || !base_type
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
        {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "generated page type must begin with a letter",
            ));
        }

        let type_name = format!("{base_type}View");

        let constant = format!("{}_INDEX", module.to_ascii_uppercase(),);

        let title = raw.to_case(Case::Title);

        Ok(Self {
            module,
            type_name,
            constant,
            title,
        })
    }

    fn page_source_path(&self, root: &Path) -> PathBuf {
        root.join("src/web/pages")
            .join(format!("{}.rs", self.module,))
    }

    fn controller_source_path(&self, root: &Path) -> PathBuf {
        root.join("src/web/controllers")
            .join(format!("{}.rs", self.module,))
    }

    fn view_model_source_path(&self, root: &Path) -> PathBuf {
        root.join("src/web/view_models")
            .join(format!("{}.rs", self.module,))
    }

    fn template_path(&self, root: &Path) -> PathBuf {
        root.join("resources/views/pages")
            .join(&self.module)
            .join("index.html")
    }

    fn template_name(&self) -> String {
        format!("pages/{}/index.html", self.module,)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoutePath(String);

impl RoutePath {
    fn parse(raw: &str) -> io::Result<Self> {
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "page route cannot be empty",
            ));
        }

        if !raw.starts_with('/') {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "page route must begin with '/'",
            ));
        }

        if raw.len() > 1 && raw.ends_with('/') {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "page route must not end with '/'",
            ));
        }

        if raw == "/" {
            return Ok(Self(raw.to_owned()));
        }

        for segment in raw[1..].split('/') {
            if segment.is_empty() {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "page route may not contain empty path segments",
                ));
            }

            if !segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '-' || character == '_'
            }) {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "make:page currently supports static route segments containing only letters, numbers, '-' and '_'",
                ));
            }
        }

        Ok(Self(raw.to_owned()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
struct GeneratedFile {
    path: PathBuf,
    contents: String,
}

#[derive(Debug)]
struct Replacement {
    path: PathBuf,
    original: String,
    updated: String,
}

#[derive(Debug)]
struct PageScaffoldPlan {
    names: PageNames,
    route: RoutePath,
    generated_files: Vec<GeneratedFile>,
    replacements: Vec<Replacement>,
    template_parent: PathBuf,
}

impl PageScaffoldPlan {
    fn prepare(project_root: &Path, names: &PageNames, route: &RoutePath) -> io::Result<Self> {
        let pages_mod_path = project_root.join("src/web/pages/mod.rs");

        let controllers_mod_path = project_root.join("src/web/controllers/mod.rs");

        let view_models_mod_path = project_root.join("src/web/view_models/mod.rs");

        let registry_path = project_root.join("src/web/views/registry.rs");

        let views_mod_path = project_root.join("src/web/views/mod.rs");

        let pages_mod = fs::read_to_string(&pages_mod_path)?;

        let controllers_mod = fs::read_to_string(&controllers_mod_path)?;

        let view_models_mod = fs::read_to_string(&view_models_mod_path)?;

        let registry = fs::read_to_string(&registry_path)?;

        let views_mod = fs::read_to_string(&views_mod_path)?;

        require_single_marker(&pages_mod, PAGE_MODULE_MARKER)?;

        require_single_marker(&controllers_mod, CONTROLLER_MODULE_MARKER)?;

        require_single_marker(&view_models_mod, VIEW_MODEL_MODULE_MARKER)?;

        require_single_marker(&view_models_mod, VIEW_MODEL_EXPORT_MARKER)?;

        require_single_marker(&registry, VIEW_CONSTANT_MARKER)?;

        require_single_marker(&registry, REGISTERED_VIEW_MARKER)?;

        require_single_marker(&views_mod, VIEW_EXPORT_MARKER)?;

        let page_source_path = names.page_source_path(project_root);

        let controller_source_path = names.controller_source_path(project_root);

        let view_model_source_path = names.view_model_source_path(project_root);

        let template_path = names.template_path(project_root);

        ensure_path_absent(&page_source_path, "page")?;

        ensure_path_absent(&controller_source_path, "controller")?;

        ensure_path_absent(&view_model_source_path, "view model")?;

        ensure_path_absent(&template_path, "view")?;

        let page_registration = format!("mod {};", names.module,);

        ensure_line_absent(&pages_mod, &page_registration, "page module", &names.module)?;

        let controller_registration = format!("pub(in crate::web) mod {};", names.module,);

        ensure_line_absent(
            &controllers_mod,
            &controller_registration,
            "controller module",
            &names.module,
        )?;

        let view_model_module = format!("mod {};", names.module,);

        ensure_line_absent(
            &view_models_mod,
            &view_model_module,
            "view-model module",
            &names.module,
        )?;

        let view_model_export = format!(
            "pub(in crate::web) use {}::{};",
            names.module, names.type_name,
        );

        ensure_line_absent(
            &view_models_mod,
            &view_model_export,
            "view-model export",
            &names.type_name,
        )?;

        let constant_needle = format!("const {}:", names.constant,);

        if registry.contains(&constant_needle) {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!("view constant is already registered: {}", names.constant,),
            ));
        }

        let template_needle = format!("ViewName::trusted(\"{}\")", names.template_name(),);

        if registry.contains(&template_needle) {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!("view path is already registered: {}", names.template_name(),),
            ));
        }

        if has_symbol(&registry, &names.constant) || has_symbol(&views_mod, &names.constant) {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!("view symbol is already registered: {}", names.constant,),
            ));
        }

        ensure_route_available(project_root, route)?;

        let updated_pages_mod = insert_before(&pages_mod, PAGE_MODULE_MARKER, &page_registration)?;

        let updated_controllers_mod = insert_before(
            &controllers_mod,
            CONTROLLER_MODULE_MARKER,
            &controller_registration,
        )?;

        let updated_view_models_mod = insert_before(
            &view_models_mod,
            VIEW_MODEL_MODULE_MARKER,
            &view_model_module,
        )?;

        let updated_view_models_mod = insert_before(
            &updated_view_models_mod,
            VIEW_MODEL_EXPORT_MARKER,
            &view_model_export,
        )?;

        let constant_source = generate_view_constant(names);

        let updated_registry = insert_before(&registry, VIEW_CONSTANT_MARKER, &constant_source)?;

        let registered_view = format!("    {},", names.constant,);

        let updated_registry =
            insert_before(&updated_registry, REGISTERED_VIEW_MARKER, &registered_view)?;

        let view_export = format!("    {},", names.constant,);

        let updated_views_mod = insert_before(&views_mod, VIEW_EXPORT_MARKER, &view_export)?;

        let template_parent = template_path
            .parent()
            .ok_or_else(|| {
                io::Error::new(
                    ErrorKind::InvalidInput,
                    "generated page template has no parent directory",
                )
            })?
            .to_path_buf();

        let pages_root = project_root.join("resources/views/pages");

        if !pages_root.is_dir() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("page view root does not exist: {}", pages_root.display(),),
            ));
        }

        Ok(Self {
            names: names.clone(),
            route: route.clone(),

            generated_files: vec![
                GeneratedFile {
                    path: page_source_path,
                    contents: generate_page_source(names, route),
                },
                GeneratedFile {
                    path: controller_source_path,
                    contents: generate_controller_source(names),
                },
                GeneratedFile {
                    path: view_model_source_path,
                    contents: generate_view_model_source(names),
                },
                GeneratedFile {
                    path: template_path,
                    contents: generate_template(names),
                },
            ],

            replacements: vec![
                Replacement {
                    path: pages_mod_path,
                    original: pages_mod,
                    updated: updated_pages_mod,
                },
                Replacement {
                    path: controllers_mod_path,
                    original: controllers_mod,
                    updated: updated_controllers_mod,
                },
                Replacement {
                    path: view_models_mod_path,
                    original: view_models_mod,
                    updated: updated_view_models_mod,
                },
                Replacement {
                    path: registry_path,
                    original: registry,
                    updated: updated_registry,
                },
                Replacement {
                    path: views_mod_path,
                    original: views_mod,
                    updated: updated_views_mod,
                },
            ],

            template_parent,
        })
    }

    fn preview(&self) {
        println!("Rusty page scaffold preview");
        println!();

        println!("page       : {}", self.names.module,);

        println!("route      : {}", self.route.as_str(),);

        println!("view model : {}", self.names.type_name,);

        println!("view       : {}", self.names.template_name(),);

        println!("constant   : {}", self.names.constant,);

        println!();

        println!("create:");

        for generated in &self.generated_files {
            println!("  {}", generated.path.display(),);
        }

        println!();

        println!("update:");

        for replacement in &self.replacements {
            println!("  {}", replacement.path.display(),);
        }

        println!();
        println!("No files changed.");
    }

    fn apply(&self) -> io::Result<()> {
        let mut created_parent = false;

        if !self.template_parent.exists() {
            fs::create_dir(&self.template_parent)?;

            created_parent = true;
        }

        let mut created_files: Vec<PathBuf> = Vec::new();

        let mut replaced_files: Vec<(PathBuf, String)> = Vec::new();

        for generated in &self.generated_files {
            if let Err(error) = write_new(&generated.path, &generated.contents) {
                return Err(rollback_after_failure(
                    error,
                    &created_files,
                    &replaced_files,
                    if created_parent {
                        Some(&self.template_parent)
                    } else {
                        None
                    },
                ));
            }

            created_files.push(generated.path.clone());
        }

        for replacement in &self.replacements {
            if let Err(error) = fs::write(&replacement.path, &replacement.updated) {
                return Err(rollback_after_failure(
                    error,
                    &created_files,
                    &replaced_files,
                    if created_parent {
                        Some(&self.template_parent)
                    } else {
                        None
                    },
                ));
            }

            replaced_files.push((replacement.path.clone(), replacement.original.clone()));
        }

        println!("Created page:");

        println!("  route      : {}", self.route.as_str(),);

        println!("  module     : {}", self.names.module,);

        println!("  view model : {}", self.names.type_name,);

        println!("  view       : {}", self.names.template_name(),);

        println!();

        println!("Generated files:");

        for generated in &self.generated_files {
            println!("  {}", generated.path.display(),);
        }

        println!();

        println!("Updated registrations:");

        for replacement in &self.replacements {
            println!("  {}", replacement.path.display(),);
        }

        Ok(())
    }
}

fn scaffold_page(
    project_root: &Path,
    names: &PageNames,
    route: &RoutePath,
    dry_run: bool,
) -> io::Result<()> {
    let plan = PageScaffoldPlan::prepare(project_root, names, route)?;

    if dry_run {
        plan.preview();
        return Ok(());
    }

    plan.apply()
}

fn generate_page_source(names: &PageNames, route: &RoutePath) -> String {
    format!(
        "use topcoat::{{\n\
         \x20   Result,\n\
         \x20   asset::asset_config,\n\
         \x20   context::{{Cx, app_context}},\n\
         \x20   router::page,\n\
         \x20   view::View,\n\
         }};\n\
         \n\
         use crate::web::{{\n\
         \x20   ViewRenderer,\n\
         \x20   assets::{{APP_CSS, BOOTSTRAP_JS, HTMX_JS, PLAIN_CSS}},\n\
         \x20   controllers,\n\
         \x20   views::{},\n\
         }};\n\
         \n\
         #[page(\"{}\")]\n\
         async fn {}(cx: &Cx) -> Result<impl View> {{\n\
         \x20   let assets = asset_config(cx);\n\
         \n\
         \x20   let model = controllers::{}::index(\n\
         \x20       assets.resolve(APP_CSS),\n\
         \x20       assets.resolve(PLAIN_CSS),\n\
         \x20       assets.resolve(BOOTSTRAP_JS),\n\
         \x20       assets.resolve(HTMX_JS),\n\
         \x20   );\n\
         \n\
         \x20   let renderer = app_context::<ViewRenderer>(cx);\n\
         \n\
         \x20   let rendered = renderer.render({}, &model)?;\n\
         \n\
         \x20   Ok(rendered.into_view(cx))\n\
         }}\n",
        names.constant,
        route.as_str(),
        names.module,
        names.module,
        names.constant,
    )
}

fn generate_controller_source(names: &PageNames) -> String {
    format!(
        "use crate::web::view_models::{};\n\
         \n\
         pub(in crate::web) fn index(\n\
         \x20   app_css_url: String,\n\
         \x20   plain_css_url: String,\n\
         \x20   bootstrap_js_url: String,\n\
         \x20   htmx_js_url: String,\n\
         ) -> {} {{\n\
         \x20   {}::new(\n\
         \x20       app_css_url,\n\
         \x20       plain_css_url,\n\
         \x20       bootstrap_js_url,\n\
         \x20       htmx_js_url,\n\
         \x20   )\n\
         }}\n",
        names.type_name, names.type_name, names.type_name,
    )
}

fn generate_view_model_source(names: &PageNames) -> String {
    format!(
        "use serde::Serialize;\n\
         \n\
         #[derive(Debug, Serialize)]\n\
         pub(in crate::web) struct {} {{\n\
         \x20   app_css_url: String,\n\
         \x20   plain_css_url: String,\n\
         \x20   bootstrap_js_url: String,\n\
         \x20   htmx_js_url: String,\n\
         }}\n\
         \n\
         impl {} {{\n\
         \x20   pub(in crate::web) fn new(\n\
         \x20       app_css_url: String,\n\
         \x20       plain_css_url: String,\n\
         \x20       bootstrap_js_url: String,\n\
         \x20       htmx_js_url: String,\n\
         \x20   ) -> Self {{\n\
         \x20       Self {{\n\
         \x20           app_css_url,\n\
         \x20           plain_css_url,\n\
         \x20           bootstrap_js_url,\n\
         \x20           htmx_js_url,\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n",
        names.type_name, names.type_name,
    )
}

fn generate_template(names: &PageNames) -> String {
    format!(
        "{{% extends \"layouts/app.html\" %}}\n\
         \n\
         {{% block title %}}{} | Atlasyr{{% endblock %}}\n\
         \n\
         {{% block content %}}\n\
         <main>\n\
         \x20   <h1>{}</h1>\n\
         </main>\n\
         {{% endblock %}}\n",
        names.title, names.title,
    )
}

fn generate_view_constant(names: &PageNames) -> String {
    format!(
        "pub(in crate::web) const {}: ViewName =\n\
         \x20   ViewName::trusted(\"{}\");",
        names.constant,
        names.template_name(),
    )
}

fn ensure_route_available(project_root: &Path, route: &RoutePath) -> io::Result<()> {
    let pages_dir = project_root.join("src/web/pages");

    for entry in fs::read_dir(&pages_dir)? {
        let entry = entry?;

        let path = entry.path();

        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }

        let source = fs::read_to_string(&path)?;

        let syntax =
            syn::parse_file(
                &source,
            )
            .map_err(
                |error| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "could not parse existing page module {} while checking route collisions: {error}",
                            path.display(),
                        ),
                    )
                },
            )?;

        for item in syntax.items {
            let Item::Fn(function) = item else {
                continue;
            };

            for attribute in function.attrs {
                if !attribute.path().is_ident("page") {
                    continue;
                }

                let Ok(existing_route) = attribute.parse_args::<LitStr>() else {
                    // POST and other explicit method forms are
                    // intentionally not equivalent to this
                    // public GET scaffold.
                    continue;
                };

                if existing_route.value() == route.as_str() {
                    return Err(io::Error::new(
                        ErrorKind::AlreadyExists,
                        format!("GET page route is already registered: {}", route.as_str(),),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn ensure_path_absent(path: &Path, description: &str) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "refusing to overwrite existing {description}: {}",
                path.display(),
            ),
        ));
    }

    Ok(())
}

fn ensure_line_absent(
    contents: &str,
    registration: &str,
    description: &str,
    name: &str,
) -> io::Result<()> {
    if contents.lines().any(|line| line.trim() == registration) {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("{description} is already registered: {name}",),
        ));
    }

    Ok(())
}

fn require_single_marker(contents: &str, marker: &str) -> io::Result<()> {
    match contents.matches(marker).count() {
        1 => Ok(()),

        0 => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("required Rusty page generator marker is missing: {marker}"),
        )),

        count => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Rusty page generator marker appears {count} times: {marker}"),
        )),
    }
}

fn insert_before(contents: &str, marker: &str, addition: &str) -> io::Result<String> {
    let position = contents.find(marker).ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("required insertion marker is missing: {marker}"),
        )
    })?;

    let mut updated = String::with_capacity(contents.len() + addition.len() + 2);

    updated.push_str(&contents[..position]);

    updated.push_str(addition);

    updated.push_str("\n\n");

    updated.push_str(&contents[position..]);

    Ok(updated)
}

fn has_symbol(contents: &str, symbol: &str) -> bool {
    contents
        .lines()
        .any(|line| line.trim().trim_end_matches(',') == symbol)
}

fn write_new(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                io::Error::new(
                    ErrorKind::AlreadyExists,
                    format!(
                        "refusing to overwrite generated page file: {}",
                        path.display(),
                    ),
                )
            } else {
                error
            }
        })?;

    file.write_all(contents.as_bytes())
}

fn rollback_after_failure(
    original_error: io::Error,
    created_files: &[PathBuf],
    replaced_files: &[(PathBuf, String)],
    created_parent: Option<&Path>,
) -> io::Error {
    let mut rollback_errors = Vec::new();

    for (path, original) in replaced_files.iter().rev() {
        if let Err(error) = fs::write(path, original) {
            rollback_errors.push(format!("restore {}: {error}", path.display(),));
        }
    }

    for path in created_files.iter().rev() {
        if let Err(error) = fs::remove_file(path) {
            if error.kind() != ErrorKind::NotFound {
                rollback_errors.push(format!("remove {}: {error}", path.display(),));
            }
        }
    }

    if let Some(parent) = created_parent {
        if let Err(error) = fs::remove_dir(parent) {
            if error.kind() != ErrorKind::NotFound {
                rollback_errors.push(format!(
                    "remove generated directory {}: {error}",
                    parent.display(),
                ));
            }
        }
    }

    if rollback_errors.is_empty() {
        io::Error::other(format!(
            "page scaffold failed and all applied changes were rolled back: {original_error}"
        ))
    } else {
        io::Error::other(format!(
            "page scaffold failed: {original_error}; rollback also encountered: {}",
            rollback_errors.join("; "),
        ))
    }
}

fn validate_identifier(value: &str, description: &str) -> io::Result<()> {
    if value.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} cannot be empty"),
        ));
    }

    let mut characters = value.chars();

    let first = characters.next().expect("checked non-empty");

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} must begin with a letter or underscore"),
        ));
    }

    if !characters.all(|character| character.is_ascii_alphanumeric() || character == '_') {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} may contain only letters, numbers, and underscores"),
        ));
    }

    if is_rust_keyword(value) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{description} may not be a reserved Rust keyword"),
        ));
    }

    Ok(())
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "yield"
    )
}

#[cfg(test)]
mod tests;
