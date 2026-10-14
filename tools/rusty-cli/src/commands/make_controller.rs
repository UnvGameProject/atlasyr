use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};

use crate::{cli::ControllerArgs, commands::CommandResult, project::Project};

const CONTROLLER_MODULE_MARKER: &str = "// RUSTY_CONTROLLER_MODULES";

pub(crate) fn run(args: ControllerArgs) -> CommandResult {
    let project = Project::discover()?;

    let names = ControllerNames::parse(&args.name)?;

    scaffold_controller(project.root(), &names, args.dry_run)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ControllerNames {
    module: String,
    display: String,
}

impl ControllerNames {
    fn parse(raw: &str) -> io::Result<Self> {
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "controller name cannot be empty",
            ));
        }

        if raw.contains('/') || raw.contains('\\') || raw.contains('.') {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "controller name must be a single module name, not a path",
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
                "controller name contains unsupported characters",
            ));
        }

        let module = raw.to_case(Case::Snake);

        validate_identifier(&module, "generated controller module")?;

        let display = raw.to_case(Case::Title);

        Ok(Self { module, display })
    }

    fn source_path(&self, project_root: &Path) -> PathBuf {
        project_root
            .join("src/web/controllers")
            .join(format!("{}.rs", self.module,))
    }

    fn registration(&self) -> String {
        format!("pub(in crate::web) mod {};", self.module,)
    }
}

fn scaffold_controller(
    project_root: &Path,
    names: &ControllerNames,
    dry_run: bool,
) -> io::Result<()> {
    let controller_path = names.source_path(project_root);

    let module_path = project_root.join("src/web/controllers/mod.rs");

    let module_source = fs::read_to_string(&module_path)?;

    require_single_marker(&module_source, CONTROLLER_MODULE_MARKER)?;

    ensure_available(names, &controller_path, &module_source)?;

    let updated_module = insert_before(
        &module_source,
        CONTROLLER_MODULE_MARKER,
        &names.registration(),
    )?;

    let controller_source = generate_controller(names);

    if dry_run {
        println!("Rusty controller scaffold preview");
        println!();
        println!("controller : {}", names.module,);
        println!("source     : {}", controller_path.display(),);
        println!("register   : {}", names.registration(),);
        println!("module     : {}", module_path.display(),);
        println!();
        println!("No files changed.");

        return Ok(());
    }

    write_new(&controller_path, &controller_source)?;

    if let Err(error) = fs::write(&module_path, updated_module) {
        let _ = fs::remove_file(&controller_path);

        return Err(io::Error::other(format!(
            "controller source was removed after module registration failed: {error}"
        )));
    }

    println!("Created controller:");
    println!("  {}", controller_path.display(),);
    println!();
    println!("Registered:");
    println!("  {}", names.registration(),);
    println!();
    println!("Updated:");
    println!("  {}", module_path.display(),);

    Ok(())
}

fn generate_controller(names: &ControllerNames) -> String {
    format!("//! Presentation controller for {}.\n", names.display,)
}

fn ensure_available(
    names: &ControllerNames,
    controller_path: &Path,
    module_source: &str,
) -> io::Result<()> {
    if controller_path.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "refusing to overwrite existing controller: {}",
                controller_path.display(),
            ),
        ));
    }

    let registration = names.registration();

    if module_source
        .lines()
        .any(|line| line.trim() == registration)
    {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("controller module is already registered: {}", names.module,),
        ));
    }

    Ok(())
}

fn require_single_marker(contents: &str, marker: &str) -> io::Result<()> {
    match contents.matches(marker).count() {
        1 => Ok(()),

        0 => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("required Rusty controller generator marker is missing: {marker}"),
        )),

        count => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Rusty controller generator marker appears {count} times: {marker}"),
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
                        "refusing to overwrite existing controller: {}",
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
