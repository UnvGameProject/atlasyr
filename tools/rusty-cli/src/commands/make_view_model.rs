use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};

use crate::{cli::ViewModelArgs, commands::CommandResult, project::Project};

const VIEW_MODEL_MODULE_MARKER: &str = "// RUSTY_VIEW_MODEL_MODULES";

const VIEW_MODEL_EXPORT_MARKER: &str = "// RUSTY_VIEW_MODEL_EXPORTS";

pub(crate) fn run(args: ViewModelArgs) -> CommandResult {
    let project = Project::discover()?;

    let names = ViewModelNames::parse(&args.name)?;

    scaffold_view_model(project.root(), &names, args.dry_run)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ViewModelNames {
    module: String,
    type_name: String,
}

impl ViewModelNames {
    fn parse(raw: &str) -> io::Result<Self> {
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "view-model name cannot be empty",
            ));
        }

        if raw.contains('/') || raw.contains('\\') || raw.contains('.') {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "view-model name must be a single module name, not a path",
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
                "view-model name contains unsupported characters",
            ));
        }

        let module = raw.to_case(Case::Snake);

        validate_identifier(&module, "generated view-model module")?;

        let base_type = raw.to_case(Case::Pascal);

        if base_type.is_empty()
            || !base_type
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
        {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "generated view-model type must begin with a letter",
            ));
        }

        let type_name = format!("{base_type}View");

        Ok(Self { module, type_name })
    }

    fn source_path(&self, project_root: &Path) -> PathBuf {
        project_root
            .join("src/web/view_models")
            .join(format!("{}.rs", self.module,))
    }

    fn module_registration(&self) -> String {
        format!("mod {};", self.module,)
    }

    fn export_registration(&self) -> String {
        format!(
            "pub(in crate::web) use {}::{};",
            self.module, self.type_name,
        )
    }
}

fn scaffold_view_model(
    project_root: &Path,
    names: &ViewModelNames,
    dry_run: bool,
) -> io::Result<()> {
    let source_path = names.source_path(project_root);

    let module_path = project_root.join("src/web/view_models/mod.rs");

    let module_source = fs::read_to_string(&module_path)?;

    require_single_marker(&module_source, VIEW_MODEL_MODULE_MARKER)?;

    require_single_marker(&module_source, VIEW_MODEL_EXPORT_MARKER)?;

    ensure_available(names, &source_path, &module_source)?;

    let updated_module = insert_before(
        &module_source,
        VIEW_MODEL_MODULE_MARKER,
        &names.module_registration(),
    )?;

    let updated_module = insert_before(
        &updated_module,
        VIEW_MODEL_EXPORT_MARKER,
        &names.export_registration(),
    )?;

    let view_model_source = generate_view_model(names);

    if dry_run {
        println!("Rusty view-model scaffold preview");
        println!();
        println!("view model : {}", names.module,);
        println!("type       : {}", names.type_name,);
        println!("source     : {}", source_path.display(),);
        println!("module     : {}", names.module_registration(),);
        println!("export     : {}", names.export_registration(),);
        println!("registry   : {}", module_path.display(),);
        println!();
        println!("No files changed.");

        return Ok(());
    }

    write_new(&source_path, &view_model_source)?;

    if let Err(error) = fs::write(&module_path, updated_module) {
        let _ = fs::remove_file(&source_path);

        return Err(io::Error::other(format!(
            "view-model source was removed after module registration failed: {error}"
        )));
    }

    println!("Created view model:");
    println!("  {}", source_path.display(),);
    println!();
    println!("Registered:");
    println!("  {}", names.module_registration(),);
    println!("  {}", names.export_registration(),);
    println!();
    println!("Updated:");
    println!("  {}", module_path.display(),);

    Ok(())
}

fn generate_view_model(names: &ViewModelNames) -> String {
    format!(
        "use serde::Serialize;\n\
         \n\
         #[derive(Debug, Serialize)]\n\
         pub(in crate::web) struct {} {{}}\n\
         \n\
         impl {} {{\n\
         \x20   pub(in crate::web) fn new() -> Self {{\n\
         \x20       Self {{}}\n\
         \x20   }}\n\
         }}\n",
        names.type_name, names.type_name,
    )
}

fn ensure_available(
    names: &ViewModelNames,
    source_path: &Path,
    module_source: &str,
) -> io::Result<()> {
    if source_path.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "refusing to overwrite existing view model: {}",
                source_path.display(),
            ),
        ));
    }

    let module_registration = names.module_registration();

    if module_source
        .lines()
        .any(|line| line.trim() == module_registration)
    {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("view-model module is already registered: {}", names.module,),
        ));
    }

    let export_registration = names.export_registration();

    if module_source
        .lines()
        .any(|line| line.trim() == export_registration)
    {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("view-model type is already exported: {}", names.type_name,),
        ));
    }

    Ok(())
}

fn require_single_marker(contents: &str, marker: &str) -> io::Result<()> {
    match contents.matches(marker).count() {
        1 => Ok(()),

        0 => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("required Rusty view-model generator marker is missing: {marker}"),
        )),

        count => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Rusty view-model generator marker appears {count} times: {marker}"),
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
                        "refusing to overwrite existing view model: {}",
                        path.display(),
                    ),
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
