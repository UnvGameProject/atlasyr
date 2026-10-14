use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{PageNames, PageScaffoldPlan, RoutePath, scaffold_page};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);

        let root = env::temp_dir().join(format!("rusty-page-{label}-{}-{id}", process::id(),));

        for directory in [
            "src/web/pages",
            "src/web/controllers",
            "src/web/view_models",
            "src/web/views",
            "resources/views/pages",
        ] {
            fs::create_dir_all(root.join(directory))
                .expect("temporary project directory should be created");
        }

        fs::write(
            root.join("src/web/pages/mod.rs"),
            "\
mod home;

// RUSTY_PAGE_MODULES
",
        )
        .expect("page registry fixture should be written");

        fs::write(
            root.join("src/web/pages/home.rs"),
            "\
use topcoat::router::page;

#[page(\"/\")]
async fn home() {}
",
        )
        .expect("existing page fixture should be written");

        fs::write(
            root.join("src/web/controllers/mod.rs"),
            "\
pub(in crate::web) mod home;

// RUSTY_CONTROLLER_MODULES
",
        )
        .expect("controller registry fixture should be written");

        fs::write(
            root.join("src/web/view_models/mod.rs"),
            "\
mod home;

// RUSTY_VIEW_MODEL_MODULES

pub(in crate::web) use home::HomeView;

// RUSTY_VIEW_MODEL_EXPORTS
",
        )
        .expect("view-model registry fixture should be written");

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
        .expect("view registry fixture should be written");

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
        .expect("view exports fixture should be written");

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
fn normalizes_page_name() {
    let names = PageNames::parse("AdminDashboard").expect("page name should normalize");

    assert_eq!(names.module, "admin_dashboard",);

    assert_eq!(names.type_name, "AdminDashboardView",);

    assert_eq!(names.constant, "ADMIN_DASHBOARD_INDEX",);

    assert_eq!(names.template_name(), "pages/admin_dashboard/index.html",);
}

#[test]
fn accepts_static_route() {
    let route = RoutePath::parse("/admin/dashboard").expect("static route should parse");

    assert_eq!(route.as_str(), "/admin/dashboard",);
}

#[test]
fn rejects_non_static_routes() {
    for invalid in [
        "dashboard",
        "/dashboard/",
        "//dashboard",
        "/dashboard/{id}",
        "/dashboard?mode=full",
        "/dashboard#content",
        r"/dashboard\child",
    ] {
        let error = RoutePath::parse(invalid).expect_err("unsupported route must be rejected");

        assert_eq!(error.kind(), ErrorKind::InvalidInput,);
    }
}

#[test]
fn scaffolds_complete_asset_aware_page() {
    let project = TempProject::new("complete");

    let names = PageNames::parse("Dashboard").expect("page name should normalize");

    let route = RoutePath::parse("/dashboard").expect("route should normalize");

    scaffold_page(project.root(), &names, &route, false).expect("complete page should scaffold");

    let page = fs::read_to_string(project.root().join("src/web/pages/dashboard.rs"))
        .expect("generated page should be readable");

    assert!(page.contains("#[page(\"/dashboard\")]",),);

    assert!(page.contains("asset_config(cx)",),);

    assert!(page.contains("assets.resolve(APP_CSS)",),);

    assert!(page.contains("renderer.render(DASHBOARD_INDEX, &model)?",),);

    let controller = fs::read_to_string(project.root().join("src/web/controllers/dashboard.rs"))
        .expect("generated controller should be readable");

    assert!(controller.contains("use crate::web::view_models::DashboardView;",),);

    assert!(controller.contains("pub(in crate::web) fn index(",),);

    let view_model = fs::read_to_string(project.root().join("src/web/view_models/dashboard.rs"))
        .expect("generated view model should be readable");

    for asset in [
        "app_css_url",
        "plain_css_url",
        "bootstrap_js_url",
        "htmx_js_url",
    ] {
        assert!(
            view_model.contains(asset,),
            "generated page view model must satisfy layout asset contract",
        );
    }

    let template = fs::read_to_string(
        project
            .root()
            .join("resources/views/pages/dashboard/index.html"),
    )
    .expect("generated template should be readable");

    assert!(template.contains("{% extends \"layouts/app.html\" %}",),);

    let pages_mod = fs::read_to_string(project.root().join("src/web/pages/mod.rs"))
        .expect("page registry should be readable");

    assert!(pages_mod.contains("mod dashboard;",),);

    let controllers_mod = fs::read_to_string(project.root().join("src/web/controllers/mod.rs"))
        .expect("controller registry should be readable");

    assert!(controllers_mod.contains("pub(in crate::web) mod dashboard;",),);

    let view_models_mod = fs::read_to_string(project.root().join("src/web/view_models/mod.rs"))
        .expect("view-model registry should be readable");

    assert!(view_models_mod.contains("mod dashboard;",),);

    assert!(view_models_mod.contains("pub(in crate::web) use dashboard::DashboardView;",),);

    let registry = fs::read_to_string(project.root().join("src/web/views/registry.rs"))
        .expect("view registry should be readable");

    assert!(registry.contains("const DASHBOARD_INDEX: ViewName",),);

    assert!(registry.contains("ViewName::trusted(\"pages/dashboard/index.html\")",),);

    let views_mod = fs::read_to_string(project.root().join("src/web/views/mod.rs"))
        .expect("view exports should be readable");

    assert!(
        views_mod
            .lines()
            .any(|line| { line.trim() == "DASHBOARD_INDEX," },),
    );
}

#[test]
fn dry_run_changes_no_files() {
    let project = TempProject::new("dry-run");

    let names = PageNames::parse("Dashboard").expect("page name should normalize");

    let route = RoutePath::parse("/dashboard").expect("route should normalize");

    let tracked = [
        "src/web/pages/mod.rs",
        "src/web/controllers/mod.rs",
        "src/web/view_models/mod.rs",
        "src/web/views/registry.rs",
        "src/web/views/mod.rs",
    ];

    let before: Vec<(String, String)> = tracked
        .iter()
        .map(|relative| {
            (
                (*relative).to_owned(),
                fs::read_to_string(project.root().join(relative))
                    .expect("fixture should be readable"),
            )
        })
        .collect();

    scaffold_page(project.root(), &names, &route, true).expect("dry run should succeed");

    for (relative, original) in before {
        assert_eq!(
            fs::read_to_string(project.root().join(&relative,),)
                .expect("fixture should remain readable",),
            original,
        );
    }

    assert!(!project.root().join("src/web/pages/dashboard.rs",).exists(),);

    assert!(
        !project
            .root()
            .join("resources/views/pages/dashboard/index.html",)
            .exists(),
    );
}

#[test]
fn refuses_existing_generated_boundary_before_writes() {
    let project = TempProject::new("existing");

    fs::write(
        project.root().join("src/web/controllers/dashboard.rs"),
        "existing\n",
    )
    .expect("existing controller fixture should be written");

    let pages_before = fs::read_to_string(project.root().join("src/web/pages/mod.rs"))
        .expect("page registry should be readable");

    let names = PageNames::parse("Dashboard").expect("page name should normalize");

    let route = RoutePath::parse("/dashboard").expect("route should normalize");

    let error = scaffold_page(project.root(), &names, &route, false)
        .expect_err("existing boundary must fail preflight");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert_eq!(
        fs::read_to_string(project.root().join("src/web/pages/mod.rs",),)
            .expect("page registry should remain readable",),
        pages_before,
    );

    assert!(!project.root().join("src/web/pages/dashboard.rs",).exists(),);
}

#[test]
fn refuses_existing_get_route_before_writes() {
    let project = TempProject::new("route-collision");

    let names = PageNames::parse("Dashboard").expect("page name should normalize");

    let route = RoutePath::parse("/").expect("root route should parse");

    let error = scaffold_page(project.root(), &names, &route, false)
        .expect_err("existing GET route must be refused");

    assert_eq!(error.kind(), ErrorKind::AlreadyExists,);

    assert!(
        error
            .to_string()
            .contains("GET page route is already registered",),
    );

    assert!(!project.root().join("src/web/pages/dashboard.rs",).exists(),);
}

#[test]
fn refuses_missing_marker_before_writes() {
    let project = TempProject::new("missing-marker");

    let pages_mod = project.root().join("src/web/pages/mod.rs");

    let source = fs::read_to_string(&pages_mod).expect("page registry fixture should be readable");

    fs::write(&pages_mod, source.replace("// RUSTY_PAGE_MODULES", ""))
        .expect("page registry fixture should be modified");

    let names = PageNames::parse("Dashboard").expect("page name should normalize");

    let route = RoutePath::parse("/dashboard").expect("route should normalize");

    let error = scaffold_page(project.root(), &names, &route, false)
        .expect_err("missing marker must fail preflight");

    assert_eq!(error.kind(), ErrorKind::InvalidData,);

    assert!(!project.root().join("src/web/pages/dashboard.rs",).exists(),);
}

#[test]
fn failed_registry_write_rolls_back_applied_changes() {
    let project = TempProject::new("rollback");

    let names = PageNames::parse("Dashboard").expect("page name should normalize");

    let route = RoutePath::parse("/dashboard").expect("route should normalize");

    let plan = PageScaffoldPlan::prepare(project.root(), &names, &route)
        .expect("page plan should preflight");

    let original_pages = fs::read_to_string(project.root().join("src/web/pages/mod.rs"))
        .expect("page registry should be readable");

    let original_controllers =
        fs::read_to_string(project.root().join("src/web/controllers/mod.rs"))
            .expect("controller registry should be readable");

    let original_view_models =
        fs::read_to_string(project.root().join("src/web/view_models/mod.rs"))
            .expect("view-model registry should be readable");

    let original_registry = fs::read_to_string(project.root().join("src/web/views/registry.rs"))
        .expect("view registry should be readable");

    let failing_path = project.root().join("src/web/views/mod.rs");

    fs::remove_file(&failing_path).expect("view exports fixture should be removable");

    fs::create_dir(&failing_path).expect("directory should force replacement write failure");

    let error = plan
        .apply()
        .expect_err("registry write failure should fail transaction");

    assert!(error.to_string().contains("rolled back",),);

    assert_eq!(
        fs::read_to_string(project.root().join("src/web/pages/mod.rs",),)
            .expect("page registry should remain readable",),
        original_pages,
    );

    assert_eq!(
        fs::read_to_string(project.root().join("src/web/controllers/mod.rs",),)
            .expect("controller registry should remain readable",),
        original_controllers,
    );

    assert_eq!(
        fs::read_to_string(project.root().join("src/web/view_models/mod.rs",),)
            .expect("view-model registry should remain readable",),
        original_view_models,
    );

    assert_eq!(
        fs::read_to_string(project.root().join("src/web/views/registry.rs",),)
            .expect("view registry should remain readable",),
        original_registry,
    );

    for generated in [
        "src/web/pages/dashboard.rs",
        "src/web/controllers/dashboard.rs",
        "src/web/view_models/dashboard.rs",
        "resources/views/pages/dashboard/index.html",
    ] {
        assert!(
            !project.root().join(generated,).exists(),
            "generated file should be removed by rollback: {generated}",
        );
    }

    assert!(
        !project
            .root()
            .join("resources/views/pages/dashboard",)
            .exists(),
        "generated template directory should be removed by rollback",
    );
}
