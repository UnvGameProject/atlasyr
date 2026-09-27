use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Component, Path, PathBuf},
};

use convert_case::{Case, Casing};

use crate::{cli::ViewArgs, commands::CommandResult, project::Project};

const VIEW_CONSTANT_MARKER: &str = "// RUSTY_VIEW_CONSTANTS";

const REGISTERED_VIEW_MARKER: &str = "    // RUSTY_REGISTERED_VIEWS";

const VIEW_EXPORT_MARKER: &str = "    // RUSTY_VIEW_EXPORTS";

pub(crate) fn run(args: ViewArgs) -> CommandResult {
    let project = Project::discover()?;

    let names = ViewNames::parse(&args.path)?;

    scaffold_view(project.root(), &names, args.dry_run)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ViewNames {
    logical: String,
    constant: String,
    title: String,
}

impl ViewNames {
    fn parse(raw: &str) -> io::Result<Self> {
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "view path cannot be empty",
            ));
        }

        let path = Path::new(raw);

        if path.is_absolute() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "view path must be relative to resources/views/pages",
            ));
        }

        let mut segments = Vec::new();
        let mut first = true;

        for component in path.components() {
            match component {
                Component::CurDir => {}

                Component::Normal(segment) => {
                    let segment = segment.to_str().ok_or_else(|| {
                        io::Error::new(ErrorKind::InvalidInput, "view path must be valid UTF-8")
                    })?;

                    // Accept either:
                    //
                    // dashboard/index
                    //
                    // or:
                    //
                    // pages/dashboard/index
                    if first && segment == "pages" {
                        first = false;
                        continue;
                    }

                    if segment.contains('.') {
                        return Err(io::Error::new(
                            ErrorKind::InvalidInput,
                            "view path must omit file extensions",
                        ));
                    }

                    let normalized = segment.to_case(Case::Snake);

                    validate_identifier(&normalized, "view path segment")?;

                    segments.push(normalized);
                    first = false;
                }

                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(io::Error::new(
                        ErrorKind::InvalidInput,
                        "view path may not escape resources/views/pages",
                    ));
                }
            }
        }

        if segments.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "view path must identify a page below resources/views/pages",
            ));
        }

        let logical = segments.join("/");

        let constant = segments.join("_").to_case(Case::Snake).to_ascii_uppercase();

        validate_identifier(&constant.to_ascii_lowercase(), "generated view constant")?;

        let title_source =
            if segments.last().is_some_and(|segment| segment == "index") && segments.len() > 1 {
                &segments[segments.len() - 2]
            } else {
                segments.last().expect("checked non-empty")
            };

        let title = title_source.to_case(Case::Title);

        Ok(Self {
            logical,
            constant,
            title,
        })
    }

    fn template_name(&self) -> String {
        format!("pages/{}.html", self.logical,)
    }

    fn template_path(&self, project_root: &Path) -> PathBuf {
        project_root
            .join("resources/views/pages")
            .join(&self.logical)
            .with_extension("html")
    }
}

fn scaffold_view(project_root: &Path, names: &ViewNames, dry_run: bool) -> io::Result<()> {
    let registry_path = project_root.join("src/web/views/registry.rs");

    let views_mod_path = project_root.join("src/web/views/mod.rs");

    let template_path = names.template_path(project_root);

    let registry = fs::read_to_string(&registry_path)?;

    let views_mod = fs::read_to_string(&views_mod_path)?;

    validate_markers(&registry, &views_mod)?;

    ensure_available(names, &template_path, &registry, &views_mod)?;

    let constant_source = generate_constant(names);

    let registered_source = format!("    {},", names.constant,);

    let export_source = format!("    {},", names.constant,);

    let registry = insert_before(&registry, VIEW_CONSTANT_MARKER, &constant_source)?;

    let registry = insert_before(&registry, REGISTERED_VIEW_MARKER, &registered_source)?;

    let views_mod = insert_before(&views_mod, VIEW_EXPORT_MARKER, &export_source)?;

    let template = generate_template(names);

    if dry_run {
        println!("Rusty view scaffold preview");
        println!();
        println!("view     : {}", names.logical,);
        println!("template : {}", template_path.display(),);
        println!("constant : {}", names.constant,);
        println!("registry : {}", registry_path.display(),);
        println!("export   : {}", views_mod_path.display(),);
        println!();
        println!("No files changed.");

        return Ok(());
    }

    let parent = template_path.parent().ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "generated view has no parent directory",
        )
    })?;

    fs::create_dir_all(parent)?;

    write_new(&template_path, &template)?;

    let original_registry = fs::read_to_string(&registry_path)?;

    if let Err(error) = fs::write(&registry_path, registry) {
        let _ = fs::remove_file(&template_path);

        return Err(error);
    }

    if let Err(error) = fs::write(&views_mod_path, views_mod) {
        let _ = fs::write(&registry_path, original_registry);

        let _ = fs::remove_file(&template_path);

        return Err(io::Error::other(format!(
            "view template and registry changes were rolled back \
                     after export registration failed: {error}"
        )));
    }

    println!("Created view:");
    println!("  {}", template_path.display(),);
    println!();
    println!("Registered:");
    println!("  view     : {}", names.template_name(),);
    println!("  constant : {}", names.constant,);
    println!();
    println!("Updated:");
    println!("  {}", registry_path.display(),);
    println!("  {}", views_mod_path.display(),);

    Ok(())
}

fn generate_constant(names: &ViewNames) -> String {
    format!(
        "pub(in crate::web) const {}: ViewName =\n\
         \x20   ViewName::trusted(\"{}\");",
        names.constant,
        names.template_name(),
    )
}

fn generate_template(names: &ViewNames) -> String {
    format!(
        "{{% extends \"layouts/app.html\" %}}\n\
         \n\
         {{% block title %}}{} | Topcoat POC{{% endblock %}}\n\
         \n\
         {{% block content %}}\n\
         <main>\n\
         \x20   <h1>{}</h1>\n\
         </main>\n\
         {{% endblock %}}\n",
        names.title, names.title,
    )
}

fn validate_markers(registry: &str, views_mod: &str) -> io::Result<()> {
    require_single_marker(registry, VIEW_CONSTANT_MARKER)?;

    require_single_marker(registry, REGISTERED_VIEW_MARKER)?;

    require_single_marker(views_mod, VIEW_EXPORT_MARKER)?;

    Ok(())
}

fn require_single_marker(contents: &str, marker: &str) -> io::Result<()> {
    match contents.matches(marker).count() {
        1 => Ok(()),

        0 => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("required Rusty view generator marker is missing: {marker}"),
        )),

        count => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Rusty view generator marker appears {count} times: {marker}"),
        )),
    }
}

fn ensure_available(
    names: &ViewNames,
    template_path: &Path,
    registry: &str,
    views_mod: &str,
) -> io::Result<()> {
    if template_path.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "refusing to overwrite existing view: {}",
                template_path.display(),
            ),
        ));
    }

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

    if has_symbol(registry, &names.constant) || has_symbol(views_mod, &names.constant) {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("view symbol is already registered: {}", names.constant,),
        ));
    }

    Ok(())
}

fn has_symbol(contents: &str, symbol: &str) -> bool {
    contents
        .lines()
        .any(|line| line.trim().trim_end_matches(',') == symbol)
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

fn write_new(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                io::Error::new(
                    ErrorKind::AlreadyExists,
                    format!("refusing to overwrite existing view: {}", path.display(),),
                )
            } else {
                error
            }
        })?;

    file.write_all(contents.as_bytes())
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
