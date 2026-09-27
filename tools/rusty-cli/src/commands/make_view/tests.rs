use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{ViewNames, scaffold_view};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);

        let root = env::temp_dir().join(format!("rusty-view-{label}-{}-{id}", process::id(),));

        fs::create_dir_all(root.join("src/web/views"))
            .expect("temporary view source directory should be created");

        fs::create_dir_all(root.join("resources/views/pages"))
            .expect("temporary page view directory should be created");

        fs::write(
            root.join("src/web/views/registry.rs"),
            "\
use super::ViewName;

pub(in crate::web) const HOME_INDEX: ViewName =
    ViewName::trusted(\"pages/home/index.html\");

// RUSTY_VIEW_CONSTANTS

pub(super) const REGISTERED_VIEWS: &[ViewName] = &[
    HOME_INDEX,
    // RUSTY_REGISTERED_VIEWS
];
",
        )
        .expect("registry fixture should be written");

        fs::write(
            root.join("src/web/views/mod.rs"),
            "\
mod registry;

pub(in crate::web) use registry::{
    HOME_INDEX,
    // RUSTY_VIEW_EXPORTS
};
",
        )
        .expect("view module fixture should be written");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn parses_page_view_path() {
    let names = ViewNames::parse("Dashboard/Index").expect("view path should normalize");

    assert_eq!(names.logical, "dashboard/index",);

    assert_eq!(names.constant, "DASHBOARD_INDEX",);

    assert_eq!(names.title, "Dashboard",);

    assert_eq!(names.template_name(), "pages/dashboard/index.html",);
}

#[test]
fn accepts_optional_pages_prefix() {
    let names = ViewNames::parse("pages/admin/users").expect("pages prefix should be accepted");

    assert_eq!(names.logical, "admin/users",);

    assert_eq!(names.constant, "ADMIN_USERS",);
}

#[test]
fn rejects_parent_traversal() {
    let error = ViewNames::parse("../../outside").expect_err("parent traversal must be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput,);
}

#[test]
fn rejects_file_extension() {
    let error =
        ViewNames::parse("dashboard/index.html").expect_err("file extension must be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput,);
}

#[test]
fn scaffolds_template_and_registrations() {
    let project = TempProject::new("create");

    let names = ViewNames::parse("dashboard/index").expect("view path should normalize");

    scaffold_view(project.root(), &names, false).expect("view should scaffold");

    let template = project
        .root()
        .join("resources/views/pages/dashboard/index.html");

    assert!(template.is_file(),);

    let template_source =
        fs::read_to_string(&template).expect("generated template should be readable");

    assert!(template_source.contains("{% extends \"layouts/app.html\" %}",),);

    assert!(template_source.contains("<h1>Dashboard</h1>",),);

    let registry = fs::read_to_string(project.root().join("src/web/views/registry.rs"))
        .expect("registry should remain readable");

    assert!(registry.contains("const DASHBOARD_INDEX: ViewName",),);

    assert!(registry.contains("ViewName::trusted(\"pages/dashboard/index.html\")",),);

    assert_eq!(
        registry
            .lines()
            .filter(|line| { line.trim() == "DASHBOARD_INDEX," },)
            .count(),
        1,
    );

    let views_mod = fs::read_to_string(project.root().join("src/web/views/mod.rs"))
        .expect("views module should remain readable");

    assert_eq!(
        views_mod
            .lines()
            .filter(|line| { line.trim() == "DASHBOARD_INDEX," },)
            .count(),
        1,
    );
}

#[test]
fn dry_run_changes_no_files() {
    let project = TempProject::new("dry-run");

    let names = ViewNames::parse("dashboard/index").expect("view path should normalize");

    let registry_path = project.root().join("src/web/views/registry.rs");

    let views_mod_path = project.root().join("src/web/views/mod.rs");

    let original_registry =
        fs::read_to_string(&registry_path).expect("registry fixture should be readable");

    let original_views_mod =
        fs::read_to_string(&views_mod_path).expect("module fixture should be readable");

    scaffold_view(project.root(), &names, true).expect("dry run should succeed");

    assert!(
        !project
            .root()
            .join("resources/views/pages/dashboard/index.html",)
            .exists(),
    );

    assert_eq!(
        fs::read_to_string(registry_path,).expect("registry should remain readable",),
        original_registry,
    );

    assert_eq!(
        fs::read_to_string(views_mod_path,).expect("module should remain readable",),
        original_views_mod,
    );
}

#[test]
fn refuses_existing_template_without_registry_changes() {
    let project = TempProject::new("existing");

    let names = ViewNames::parse("dashboard/index").expect("view path should normalize");

    let template = project
        .root()
        .join("resources/views/pages/dashboard/index.html");

    fs::create_dir_all(template.parent().expect("template should have a parent"))
        .expect("template directory should be created");

    fs::write(&template, "existing\n").expect("existing template should be written");

    let registry_path = project.root().join("src/web/views/registry.rs");

    let before = fs::read_to_string(&registry_path).expect("registry should be readable");

    let error = scaffold_view(project.root(), &names, false)
        .expect_err("existing template must be refused");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert_eq!(
        fs::read_to_string(registry_path,).expect("registry should remain readable",),
        before,
    );
}

#[test]
fn refuses_missing_generator_marker_before_creating_template() {
    let project = TempProject::new("marker");

    let registry_path = project.root().join("src/web/views/registry.rs");

    let registry = fs::read_to_string(&registry_path).expect("registry fixture should be readable");

    fs::write(
        &registry_path,
        registry.replace("// RUSTY_VIEW_CONSTANTS", ""),
    )
    .expect("registry fixture should be modified");

    let names = ViewNames::parse("dashboard/index").expect("view path should normalize");

    let error = scaffold_view(project.root(), &names, false).expect_err("missing marker must fail");

    assert_eq!(error.kind(), ErrorKind::InvalidData,);

    assert!(
        !project
            .root()
            .join("resources/views/pages/dashboard/index.html",)
            .exists(),
    );
}
