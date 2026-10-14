use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{ControllerNames, scaffold_controller};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);

        let root =
            env::temp_dir().join(format!("rusty-controller-{label}-{}-{id}", process::id(),));

        fs::create_dir_all(root.join("src/web/controllers"))
            .expect("temporary controller directory should be created");

        fs::write(
            root.join("src/web/controllers/mod.rs"),
            "\
pub(in crate::web) mod home;

// RUSTY_CONTROLLER_MODULES
",
        )
        .expect("controller module fixture should be written");

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
fn normalizes_controller_name() {
    let names = ControllerNames::parse("AdminDashboard").expect("controller name should normalize");

    assert_eq!(names.module, "admin_dashboard",);

    assert_eq!(names.display, "Admin Dashboard",);
}

#[test]
fn rejects_controller_paths() {
    for invalid in [
        "../dashboard",
        "admin/dashboard",
        r"admin\dashboard",
        "dashboard.rs",
    ] {
        let error = ControllerNames::parse(invalid).expect_err("controller path must be rejected");

        assert_eq!(error.kind(), ErrorKind::InvalidInput,);
    }
}

#[test]
fn rejects_reserved_keyword() {
    let error = ControllerNames::parse("Self").expect_err("reserved keyword must be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput,);
}

#[test]
fn scaffolds_controller_and_registration() {
    let project = TempProject::new("create");

    let names = ControllerNames::parse("Dashboard").expect("controller should normalize");

    scaffold_controller(project.root(), &names, false).expect("controller should scaffold");

    let source_path = project.root().join("src/web/controllers/dashboard.rs");

    assert!(source_path.is_file(),);

    let source = fs::read_to_string(source_path).expect("controller should be readable");

    assert_eq!(source, "//! Presentation controller for Dashboard.\n",);

    let module = fs::read_to_string(project.root().join("src/web/controllers/mod.rs"))
        .expect("module should be readable");

    assert_eq!(
        module
            .lines()
            .filter(|line| { line.trim() == "pub(in crate::web) mod dashboard;" },)
            .count(),
        1,
    );
}

#[test]
fn dry_run_changes_no_files() {
    let project = TempProject::new("dry-run");

    let names = ControllerNames::parse("Dashboard").expect("controller should normalize");

    let module_path = project.root().join("src/web/controllers/mod.rs");

    let before = fs::read_to_string(&module_path).expect("module should be readable");

    scaffold_controller(project.root(), &names, true).expect("dry run should succeed");

    assert!(
        !project
            .root()
            .join("src/web/controllers/dashboard.rs",)
            .exists(),
    );

    assert_eq!(
        fs::read_to_string(module_path,).expect("module should remain readable",),
        before,
    );
}

#[test]
fn refuses_existing_controller_without_registry_changes() {
    let project = TempProject::new("existing");

    let names = ControllerNames::parse("Dashboard").expect("controller should normalize");

    let source_path = project.root().join("src/web/controllers/dashboard.rs");

    fs::write(&source_path, "existing\n").expect("existing controller should be written");

    let module_path = project.root().join("src/web/controllers/mod.rs");

    let before = fs::read_to_string(&module_path).expect("module should be readable");

    let error = scaffold_controller(project.root(), &names, false)
        .expect_err("existing controller must be refused");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert_eq!(
        fs::read_to_string(module_path,).expect("module should remain readable",),
        before,
    );
}

#[test]
fn refuses_registered_controller_without_creating_file() {
    let project = TempProject::new("registered");

    let module_path = project.root().join("src/web/controllers/mod.rs");

    let module = fs::read_to_string(&module_path).expect("module fixture should be readable");

    fs::write(
        &module_path,
        module.replace(
            "// RUSTY_CONTROLLER_MODULES",
            "pub(in crate::web) mod dashboard;\n\n// RUSTY_CONTROLLER_MODULES",
        ),
    )
    .expect("registered fixture should be written");

    let names = ControllerNames::parse("Dashboard").expect("controller should normalize");

    let error = scaffold_controller(project.root(), &names, false)
        .expect_err("registered controller must be refused");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert!(
        !project
            .root()
            .join("src/web/controllers/dashboard.rs",)
            .exists(),
    );
}

#[test]
fn refuses_missing_marker_before_creating_file() {
    let project = TempProject::new("marker");

    let module_path = project.root().join("src/web/controllers/mod.rs");

    let module = fs::read_to_string(&module_path).expect("module fixture should be readable");

    fs::write(
        &module_path,
        module.replace("// RUSTY_CONTROLLER_MODULES", ""),
    )
    .expect("module fixture should be modified");

    let names = ControllerNames::parse("Dashboard").expect("controller should normalize");

    let error =
        scaffold_controller(project.root(), &names, false).expect_err("missing marker must fail");

    assert_eq!(error.kind(), ErrorKind::InvalidData,);

    assert!(
        !project
            .root()
            .join("src/web/controllers/dashboard.rs",)
            .exists(),
    );
}
