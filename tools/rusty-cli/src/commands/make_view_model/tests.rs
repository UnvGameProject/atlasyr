use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{ViewModelNames, scaffold_view_model};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);

        let root =
            env::temp_dir().join(format!("rusty-view-model-{label}-{}-{id}", process::id(),));

        fs::create_dir_all(root.join("src/web/view_models"))
            .expect("temporary view-model directory should be created");

        fs::write(
            root.join("src/web/view_models/mod.rs"),
            "\
mod home;

// RUSTY_VIEW_MODEL_MODULES

pub(in crate::web) use home::HomeView;

// RUSTY_VIEW_MODEL_EXPORTS
",
        )
        .expect("view-model module fixture should be written");

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
fn normalizes_view_model_name() {
    let names = ViewModelNames::parse("AdminDashboard").expect("view-model name should normalize");

    assert_eq!(names.module, "admin_dashboard",);

    assert_eq!(names.type_name, "AdminDashboardView",);
}

#[test]
fn rejects_view_model_paths() {
    for invalid in [
        "../dashboard",
        "admin/dashboard",
        r"admin\dashboard",
        "dashboard.rs",
    ] {
        let error = ViewModelNames::parse(invalid).expect_err("view-model path must be rejected");

        assert_eq!(error.kind(), ErrorKind::InvalidInput,);
    }
}

#[test]
fn rejects_reserved_keyword() {
    let error = ViewModelNames::parse("Self").expect_err("reserved keyword must be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput,);
}

#[test]
fn scaffolds_view_model_and_registrations() {
    let project = TempProject::new("create");

    let names = ViewModelNames::parse("Dashboard").expect("view model should normalize");

    scaffold_view_model(project.root(), &names, false).expect("view model should scaffold");

    let source_path = project.root().join("src/web/view_models/dashboard.rs");

    assert!(source_path.is_file(),);

    let source = fs::read_to_string(source_path).expect("generated view model should be readable");

    assert!(source.contains("use serde::Serialize;",),);

    assert!(source.contains("#[derive(Debug, Serialize)]",),);

    assert!(source.contains("pub(in crate::web) struct DashboardView {}",),);

    assert!(source.contains("pub(in crate::web) fn new() -> Self",),);

    let module = fs::read_to_string(project.root().join("src/web/view_models/mod.rs"))
        .expect("view-model module should be readable");

    assert_eq!(
        module
            .lines()
            .filter(|line| { line.trim() == "mod dashboard;" },)
            .count(),
        1,
    );

    assert_eq!(
        module
            .lines()
            .filter(|line| { line.trim() == "pub(in crate::web) use dashboard::DashboardView;" },)
            .count(),
        1,
    );
}

#[test]
fn dry_run_changes_no_files() {
    let project = TempProject::new("dry-run");

    let names = ViewModelNames::parse("Dashboard").expect("view model should normalize");

    let module_path = project.root().join("src/web/view_models/mod.rs");

    let before = fs::read_to_string(&module_path).expect("module fixture should be readable");

    scaffold_view_model(project.root(), &names, true).expect("dry run should succeed");

    assert!(
        !project
            .root()
            .join("src/web/view_models/dashboard.rs",)
            .exists(),
    );

    assert_eq!(
        fs::read_to_string(module_path,).expect("module should remain readable",),
        before,
    );
}

#[test]
fn refuses_existing_view_model_without_registry_changes() {
    let project = TempProject::new("existing");

    let names = ViewModelNames::parse("Dashboard").expect("view model should normalize");

    let source_path = project.root().join("src/web/view_models/dashboard.rs");

    fs::write(&source_path, "existing\n").expect("existing view model should be written");

    let module_path = project.root().join("src/web/view_models/mod.rs");

    let before = fs::read_to_string(&module_path).expect("module fixture should be readable");

    let error = scaffold_view_model(project.root(), &names, false)
        .expect_err("existing view model must be refused");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert_eq!(
        fs::read_to_string(module_path,).expect("module should remain readable",),
        before,
    );
}

#[test]
fn refuses_registered_view_model_without_creating_file() {
    let project = TempProject::new("registered");

    let module_path = project.root().join("src/web/view_models/mod.rs");

    let module = fs::read_to_string(&module_path).expect("module fixture should be readable");

    fs::write(
        &module_path,
        module.replace(
            "// RUSTY_VIEW_MODEL_MODULES",
            "mod dashboard;\n\n// RUSTY_VIEW_MODEL_MODULES",
        ),
    )
    .expect("registered fixture should be written");

    let names = ViewModelNames::parse("Dashboard").expect("view model should normalize");

    let error = scaffold_view_model(project.root(), &names, false)
        .expect_err("registered view model must be refused");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert!(
        !project
            .root()
            .join("src/web/view_models/dashboard.rs",)
            .exists(),
    );
}

#[test]
fn refuses_missing_marker_before_creating_file() {
    let project = TempProject::new("marker");

    let module_path = project.root().join("src/web/view_models/mod.rs");

    let module = fs::read_to_string(&module_path).expect("module fixture should be readable");

    fs::write(
        &module_path,
        module.replace("// RUSTY_VIEW_MODEL_EXPORTS", ""),
    )
    .expect("module fixture should be modified");

    let names = ViewModelNames::parse("Dashboard").expect("view model should normalize");

    let error =
        scaffold_view_model(project.root(), &names, false).expect_err("missing marker must fail");

    assert_eq!(error.kind(), ErrorKind::InvalidData,);

    assert!(
        !project
            .root()
            .join("src/web/view_models/dashboard.rs",)
            .exists(),
    );
}
