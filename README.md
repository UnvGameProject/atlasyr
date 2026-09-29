
# Topcoat POC

A security-first, Dockerized proof of concept for evaluating **Topcoat** as a Rust frontend/full-stack framework against a mature **Laravel + Livewire-style** development baseline.

This repository is also intended to become a reusable GitHub template for future Topcoat applications if the framework proves suitable.

## Purpose

The goal is not simply to prove that Rust can render a web application.

The POC is evaluating whether Topcoat can provide a productive, maintainable, and secure application-development experience while allowing an existing Rust-heavy backend to remain in Rust.

Laravel and mature industry implementations are used as reference points whenever Topcoat does not yet provide an equivalent built-in capability.

The evaluation prioritizes:

1. **Security**
2. **Organization and maintainability**
3. **Developer experience**
4. **Framework maturity**
5. **Operational simplicity**
6. **Frontend flexibility**
7. **Performance**

## Getting started

### Prerequisites

Install:

- Docker with Docker Compose
- Git
- Rustup
- Rust 1.98.1, or allow `rust-toolchain.toml` to select/install it automatically
- OpenSSL for generating local PostgreSQL passwords

The project uses Docker for the application runtime, frontend asset build, PostgreSQL, isolated database testing, and Rusty CLI tooling. A host Rust toolchain is also supported for IDE integration, formatting, linting, and local `cargo check`.

### Clone the repository

```bash
git clone <your-repository-url>
cd topcoat_poc
```

If this repository is being used as a GitHub template, create a new repository from the template first, then clone the new repository.

### Create the local development secrets

The preferred development configuration uses Docker Compose secrets rather than a `.env` file.

The PostgreSQL setup separates bootstrap, migration, runtime application, and isolated test privileges. Create four local passwords:

```bash
mkdir -p .secrets
chmod 700 .secrets

openssl rand -hex 24 > .secrets/postgres_bootstrap_password
openssl rand -hex 24 > .secrets/postgres_migrator_password
openssl rand -hex 24 > .secrets/postgres_app_password
openssl rand -hex 24 > .secrets/postgres_test_password

chmod 644 .secrets/postgres_*_password
```

The `.secrets/` directory itself remains `0700`, preventing other host users from traversing it. The individual secret files use `0644` because Docker Compose file-backed secrets preserve host-file permissions and PostgreSQL initialization runs as the container's non-root `postgres` OS user.

The `.secrets/` directory is ignored by Git and must never be committed.

The roles are intentionally separated:

- `postgres` — bootstrap superuser used only by the normal PostgreSQL container during initialization and administration.
- `topcoat_migrator` — non-superuser schema owner used for application migrations.
- `topcoat_app` — least-privilege runtime user used by the Topcoat application.
- `topcoat_test_admin` — test-only PostgreSQL administrator used only inside the isolated `postgres-test` environment so SQLx can create and destroy per-test databases.

The normal application container receives only the runtime application's password secret. The test runner receives only the test password secret.

Because the repository is bind-mounted into several development containers, `.secrets/` is additionally masked with container-local `tmpfs` mounts. Services must consume granted secrets from `/run/secrets/...`, not from the repository bind mount.

### Build and start the application

Build and start the normal development stack:

```bash
docker compose up --build
```

Or, after the images have already been built:

```bash
docker compose up -d
```

The normal development stack includes PostgreSQL, the frontend asset builder, the Topcoat application, the migration runner, and the Rusty CLI development-tool container.

The isolated test services use the Compose `test` profile and are intentionally excluded from ordinary startup. They are started automatically when the canonical project test command is used.

Check normal service status:

```bash
docker compose ps
```

The application should be available at:

```text
http://localhost:3000
```

The development port is intentionally bound to `127.0.0.1`, so it is not exposed to the LAN by default.

### Verify the Rust toolchain

On the host:

```bash
rustup show
rustc --version
cargo --version
```

The repository pins the development toolchain with `rust-toolchain.toml`.

Verify the application locally:

```bash
cargo check --locked
```

Verify the Docker build environment:

```bash
docker compose exec app rustc --version
docker compose exec app cargo check --locked
```

### Verify PostgreSQL

Once the PostgreSQL service is present and started:

```bash
docker compose exec postgres sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT current_database(), current_user, version();"'
```

Verify that the application container can read its runtime Docker secret without printing the password:

```bash
docker compose exec app sh -c \
  'test -r /run/secrets/postgres_app_password && echo "postgres_app_password secret: readable"'
```

The runtime application should not receive the bootstrap or migrator password files.

### Verify Rusty CLI

Rusty CLI is the project-local Rust developer tool used for scaffolding and project maintenance.

Build and start it:

```bash
docker compose build rusty
docker compose up -d rusty
```

Verify it from the repository root:

```bash
./rusty --version
./rusty doctor
```

The repository-root `rusty` file is a location-independent shell wrapper at:

```text
<repository-root>/rusty
```

It resolves the repository root from its own location, anchors Docker Compose to this project's `compose.yaml`, starts the Rusty service when needed, and executes the containerized CLI with the host UID/GID so generated project files are not created as `root`.

Because the wrapper is location-independent, an optional Bash alias can make Rusty available globally:

```bash
grep -Fqx \
  'alias rusty="$HOME/projects/topcoat_poc/rusty"' \
  ~/.bashrc || cat >> ~/.bashrc <<'EOF'

# Topcoat POC Rusty CLI
alias rusty="$HOME/projects/topcoat_poc/rusty"
EOF

source ~/.bashrc
```

Verify the alias:

```bash
type rusty
rusty --version
rusty doctor
```

The alias continues to work when the shell is outside the repository.

Create a reversible SQLx migration:

```bash
rusty migration CreateContacts
```

The compatibility alias is also available:

```bash
rusty make:migration CreateContacts
```

For an intentionally irreversible migration:

```bash
rusty migration SomeOneWayChange --simple
```

Scaffold a module-local test file:

```bash
rusty make:test db/models/contact validation
```

This creates the test under the target module's `tests/` directory and updates the relevant `mod.rs` registrations without overwriting an existing test file.

Preview a new Rusty command scaffold without changing files:

```bash
rusty make:command model:sync --dry-run
```

`make:command` uses deterministic generator markers in Rusty's own command registry so new CLI commands can be scaffolded without manually wiring every module, enum variant, and dispatcher entry.

### Development logs

Follow the Topcoat application logs:

```bash
docker compose logs -f app
```

Follow only recent application logs:

```bash
docker compose logs --since=1m -f app
```

Follow frontend asset-builder logs:

```bash
docker compose logs -f assets
```

### Formatting and linting

Check formatting:

```bash
cargo fmt --check
```

Run Clippy with warnings treated as errors:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Run the same checks in Docker when needed:

```bash
docker compose exec app cargo fmt --check
docker compose exec app cargo clippy --all-targets --all-features -- -D warnings
```

### Testing

The canonical project-wide test entrypoint is:

```bash
./test
```

`./test` starts or reuses the isolated `postgres-test` service, invokes the test runner inside its dedicated container environment, and ultimately executes Cargo's native test runner:

```text
./test
    ↓
Docker test safety gate
    ↓
cargo test --locked --all-targets
```

`cargo test` remains the underlying Rust test runner. Rusty does not execute tests; `rusty make:test` only scaffolds test files.

Pure unit tests can remain ordinary Rust tests with no database dependency. Persistence tests that require PostgreSQL use `#[sqlx::test]`.

The test architecture deliberately separates database authority:

- `postgres-test` is a separate PostgreSQL server from the normal development database.
- it uses the Compose `test` profile and does not start during normal `docker compose up`.
- it uses a test-only credential that is never mounted into the application or migration services.
- the test runner does not receive bootstrap, migrator, or runtime application database passwords.
- the test runner validates `APP_ENV`, database host, port, database name, user, secret path, and absence of non-test secrets before invoking Cargo.
- test PostgreSQL storage is backed by `tmpfs`, not the normal development data volume.
- SQLx creates an isolated logical database for each `#[sqlx::test]` database test and applies the project migrations.

The first test run may compile the test target. The dedicated `/cargo-test-target` volume keeps that work cached, and the `postgres-test` server remains warm for subsequent runs. During the POC, the verified warm full-suite run completed in approximately 1.3 seconds.

### Stop the project

```bash
docker compose down
```

To also remove project data volumes:

```bash
docker compose down -v
```

Use `-v` deliberately: it removes named volumes, including PostgreSQL development data and Cargo/Node caches.

## Current status

### POC 0 — Green

The initial frontend/runtime baseline is working.

Verified:

- Rust 1.98.1 development toolchain
- Topcoat 0.9.0
- Dockerized development environment
- direct Topcoat HTTP serving on localhost
- server-side rendering
- Topcoat layouts and routing
- Topcoat browser runtime
- Rust-authored client reactivity
- Topcoat asset bundling
- plain CSS
- SCSS
- Bootstrap 5.3.8
- locally built and self-hosted Bootstrap JavaScript
- Node isolated to the asset-builder container
- host and container Cargo build directories isolated from each other
- `cargo check --locked` succeeds both on the host and in Docker

The application is available at:

```text
http://localhost:3000
```

The host mapping intentionally binds to `127.0.0.1`, so the development application is not exposed to the LAN by default.

### POC 1 — Green application/data baseline

The application/data foundation and first end-to-end Contacts feature slice are working.

Verified:

- PostgreSQL 18.6 container
- Docker Compose file-backed secrets
- separate bootstrap, migration, runtime, and test PostgreSQL credentials
- runtime SQLx pool connected as `topcoat_app`
- dedicated `topcoat_migrator` schema-owner role
- one-shot migration runner that must complete successfully before the application starts
- SQLx native reversible migrations
- migration hash stability through LF-pinned SQL files
- SQLx offline metadata under `.sqlx/`
- compile-time Contact row-shape and nullability contracts
- Contact create/read/update/delete persistence
- application-layer Contact create/update/delete boundaries
- dedicated Contact list query/read-model boundary
- typed validation and UUID parsing before persistence
- isolated database-backed persistence tests
- separate warm `postgres-test` service
- fail-closed database test safety gate
- test-only ephemeral PostgreSQL storage
- project-wide `./test` entrypoint
- Rusty CLI standalone crate and Docker service
- reversible migration, module-aware test, and command scaffolding
- generated project files owned by the host developer rather than root

### POC 2 — Green validation/forms baseline

The first application-facing validation and mutation workflow is complete enough for the POC baseline.

Verified:

- typed Contact create/update inputs
- field-specific normalization
- validation before application/persistence work
- UUID parsing at the application boundary
- canonical HTML-first create/update/delete routes routed through the application layer
- safe browser-facing failure states rather than persistence errors leaking directly to the client
- Contact presentation migrated to HTML-first pages, controllers, view models, and runtime views; the temporary native control is retired

### POC 3 — Green authentication/session baseline

Authentication is complete for the current POC scope.

Verified:

- startup-selected authentication provider configuration
- `local-topcoat` provider with no request-selectable driver and no fallback/downgrade path
- Argon2id password hashing and verification
- generic invalid-credential responses
- unknown-user password verification work to reduce credential-enumeration timing differences
- login throttling before expensive password verification
- persisted server-side sessions using Topcoat session infrastructure
- login, logout, authenticated navigation, and protected-route enforcement
- session persistence across refreshes
- session lifetime profile with idle, absolute, and refresh windows
- explicit viewer/operator provisioning through the project CLI flow without placing passwords in argv or environment variables
- automated authentication/session lifecycle coverage

Public self-registration, password reset, email verification, and MFA are not part of the completed baseline and remain separate future capabilities.

### Rusty security audit — Green baseline

Rusty now includes a project-local security/audit pass for important Rust input and persistence boundaries.

The current checkpoint audits validated domain input, UUID parsing, credential verification, and persistence sink flows. At `auth-cleanup-final-v1`, the audit is clean with no warnings or errors.

### View/DX Architecture v1 — Runtime foundation green

View/DX Architecture v1 is adopted, and its first real application slice is green.

Verified in the repository:

- MiniJinja 2.24.0 is the v1 runtime renderer for ordinary external HTML views
- a project-owned `ViewRenderer` centralizes template rendering
- application-owned `ViewName` values prevent request-controlled template paths
- presentation data crosses the renderer through typed serializable view models
- `RenderedHtml` owns the single application-controlled bridge to Topcoat `Unescaped`
- MiniJinja HTML autoescaping remains enabled for ordinary model values
- strict undefined-value behavior fails closed instead of silently rendering missing data
- `|safe` is not used by application templates
- development template reload uses MiniJinja autoreload support rather than rebuilding the Rust application for ordinary HTML edits
- the public home page is rendered from `resources/views/pages/home/index.html`
- the external home page uses `resources/views/layouts/app.html`
- a real runtime HTML edit was observed on a subsequent request and restored without a Rust rebuild
- the existing native Topcoat login and Contacts pages remain available under logical route groups without changing their public URLs
- unauthenticated HTTP behavior remains `/` = 200, `/login` = 200, `/contacts` = 401
- native Topcoat remains available for interactions where its typed reactive model provides meaningful value

The public home page was intentionally migrated first so the renderer foundation could be proven without disturbing Contacts, authentication mutations, or other security-sensitive application functionality.

The runtime-mode hardening slice is also green. Renderer behavior is now selected explicitly through `APP_ENV` rather than Rust compilation profile:

- `development` uses MiniJinja autoreload and rich server-side diagnostics
- `testing` uses a deterministic cached renderer with startup validation
- `production` uses a cached renderer, validates registered views before the HTTP server starts, disables MiniJinja rich debug mode, and restricts renderer logging to production-safe structured diagnostics
- renderer failures return Topcoat's generic HTTP 500 response rather than exposing template source, filesystem paths, or internal model details
- registered application views are validated through the project-owned view registry

The first HTML-first htmx interaction is now green. The application pins and self-hosts htmx, exposes it through Topcoat's asset system, and serves a standalone MiniJinja fragment from `/view-dx/demo`. The route uses Topcoat's `hx_request` accessor only as request/presentation metadata; direct requests remain valid and `HX-Request` is not an authentication or authorization boundary.

Contacts migration onto the HTML-first architecture now has a protected full-page read path plus a protected htmx list-fragment path. `/view-dx/contacts` renders through a typed MiniJinja page model, while `/view-dx/contacts/list` renders the same `fragments/contacts/list.html` fragment used by the full page. Both paths reuse the existing `application::contacts::list_contacts` query, and every externally callable Contacts read route independently enforces authentication. The native `/contacts` CRUD workflow remains unchanged.

Contact creation is now available through the HTML-first path as the first htmx mutation. `POST /view-dx/contacts/create` uses Topcoat's typed form extractor, independently enforces authentication, remains protected by Topcoat's default cross-origin mutation policy, and reuses the existing `CreateContactCommand` / `create_contact` application use case. Validation and duplicate-email outcomes are mapped into typed presentation state; htmx receives the Contacts workspace fragment while ordinary form submission receives the full Contacts page. The native `/contacts` CRUD workflow remains available as the control implementation.

Contact update is now available through the HTML-first path. Each contact renders through a reusable `fragments/contacts/row.html` fragment, htmx update requests replace only the affected contact `<tbody>`, and ordinary form submission retains a full-page fallback. The update HTTP route independently enforces authentication and Topcoat's default cross-origin mutation protection while reusing the existing `UpdateContactCommand` / `update_contact` application use case. Validation, duplicate-email, not-found, and success outcomes are mapped into typed row presentation state. A focused `find_contact` application query now provides single-contact reads without exposing persistence to the web layer. During migration, the native Topcoat row used the existing `#[procedure]` update path, demonstrating that native procedures and HTML/htmx transports could coexist over the same application use case.

Contact delete is now available through the HTML-first path, completing CRUD parity for Contacts. The delete flow requires explicit confirmation, independently enforces authentication and Topcoat's default cross-origin mutation protection, and reuses the existing ID-only `DeleteContactCommand` / `delete_contact` application use case. Successful htmx deletion removes only the affected contact fragment, while not-found and infrastructure failures remain row-local; ordinary form submission retains a full-page fallback. Presentation-only name/email snapshots are used solely to reconstruct a row if persistence itself fails and never enter the delete application command. During migration, the native Topcoat delete path remained available through its existing `#[procedure]` implementation.

HTML-first Contacts is the canonical and sole Contacts presentation surface. Its list, row, create, update, and delete endpoints all live under `/contacts/*`; the temporary `/view-dx/contacts*` migration routes remain retired rather than being retained as duplicate mutation aliases. Existing application navigation and successful login flows target `/contacts`. The temporary `/native/contacts` control route has now been retired after CRUD parity, canonical routing, and progressive-enhancement behavior were established; `view-contacts-prg-v1` is the rollback point immediately before native Contacts retirement.

Contacts progressive enhancement now uses POST/Redirect/GET semantics for successful ordinary form mutations. Successful non-htmx create, update, and delete requests return Topcoat's framework-native `303 See Other` response targeting `/contacts`, preventing browser refresh from replaying mutation POSTs. htmx requests continue to receive the same workspace or row fragments as before, while validation, duplicate-email, not-found, and infrastructure-error outcomes continue rendering their typed error state without redirecting.

The temporary native Contacts presentation has now been retired together with its Contacts-specific compiled view, component, shard, and procedure tree after dependency recon confirmed those modules had no consumers outside the control implementation. This does not retire native Topcoat as an application capability: procedures, components, shards, signals, and other reactive features remain valid opt-in tools wherever their model provides value. Rusty view scaffolding and broader frontend-source reorganization remain separate later slices.

## Current stack

### Application

- Rust 1.98.1
- Rust edition 2024
- Topcoat 0.9.0
- Tokio
- SQLx 0.9.0
- PostgreSQL 18.6
- application-owned authentication with Topcoat-backed sessions

### Frontend

Committed/current:

- MiniJinja 2.24.0 runtime external HTML views
- MiniJinja autoreload support for development-time template edits
- Topcoat `view!`
- Topcoat reactive runtime
- Bootstrap 5.3.8
- Dart Sass
- plain CSS
- ordinary JavaScript through the frontend asset pipeline

Evaluated successfully in disposable View/DX spikes:

- Tera 2.4.0
- Topcoat's official htmx integration
- mixed HTML-first and native Topcoat interaction models

MiniJinja is the View/DX Architecture v1 renderer. htmx 2.0.10 is now a pinned, locally served application dependency for HTML-first server interactions where partial DOM replacement is appropriate; native Topcoat remains available for interactions that benefit from its typed reactive model.

Tailwind is supported by Topcoat but is intentionally **not required** by this template.

The frontend architecture must remain compatible with standards-based HTML/CSS/JavaScript and must not depend on any single CSS or browser-interaction framework.

### Infrastructure

- Docker Compose
- Rust application container
- dedicated frontend asset-builder container
- PostgreSQL 18.6 development database
- dedicated one-shot migration runner
- Dockerized Rusty CLI development-tool service
- isolated `postgres-test` PostgreSQL service under the Compose `test` profile
- one-shot test runner container
- dedicated Cargo target volume for test builds
- Docker Compose secrets
- separate PostgreSQL bootstrap, migration, runtime, and test credentials
- `tmpfs` masking of repository `.secrets/` inside bind-mounted containers
- ephemeral `tmpfs` storage for the isolated test PostgreSQL cluster

## Rust toolchain

The repository uses `rust-toolchain.toml` to pin the development toolchain:

```toml
[toolchain]
channel = "1.98.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

`Cargo.toml` declares:

```toml
rust-version = "1.98"
```

These serve different purposes:

- `rust-version` is the minimum supported Rust version.
- `rust-toolchain.toml` pins the exact development toolchain used by the project.

The Docker image also installs `rustfmt` and `clippy` during image construction rather than allowing the running container to mutate its Rust installation.

## Build isolation

Host Cargo, application-container Cargo, and test-container Cargo intentionally use separate target directories.

Host:

```text
./target/
```

Application/migration containers:

```text
/cargo-target/
```

Isolated test runner:

```text
/cargo-test-target/
```

The Docker target directories are backed by named Docker volumes.

This prevents container-created root-owned build artifacts from interfering with local Cargo, rust-analyzer, or IDE tooling while still preserving fast incremental rebuilds. Test compilation is cached independently so repeated `./test` runs do not contend with the Topcoat development process.

## Frontend asset policy

Plain CSS, SCSS, Bootstrap, ordinary JavaScript, and other appropriate standards-based frontend technologies are first-class options.

Current behavior:

- SCSS is compiled by a dedicated Node asset container.
- Bootstrap is installed as a build dependency.
- Bootstrap CSS is compiled locally from SCSS.
- Bootstrap JavaScript is copied into generated assets.
- Browser assets are served locally by Topcoat.
- Browser clients do not depend on a third-party Bootstrap CDN.
- Node is not part of the Rust application runtime.
- `package-lock.json` is committed for reproducible frontend builds.
- Generated frontend output is not committed.

Topcoat and the application architecture must not become dependent on Tailwind.

## View and frontend DX direction

The project uses an **HTML-first, not HTML-only** presentation model for ordinary presentation work, introduced incrementally so existing native Topcoat functionality can remain in place while each boundary is proven.

The intended separation is:

```text
src/
├── web/
│   ├── routes/
│   ├── controllers/
│   ├── view_models/
│   ├── procedures/
│   ├── components/       # native Topcoat where justified
│   └── shards/           # native Topcoat where justified
│
├── application/
├── domain/
└── db/

resources/
├── views/
│   ├── layouts/
│   ├── pages/
│   ├── partials/
│   └── fragments/
│
├── js/
├── scss/
├── css/
└── images/
```

View/DX Architecture v1 reserves external HTML terminology such as views, layouts, partials, fragments, controllers, and view models for the HTML-first path. Topcoat-specific terms such as components, procedures, shards, and signals remain reserved for native Topcoat capabilities.

The normal HTML-first page flow is:

```text
Topcoat route
    ↓
Rust controller
    ↓
application/query logic
    ↓
typed view model
    ↓
runtime HTML view
    ↓
Topcoat HTTP response
```

Ordinary page markup, styling, and browser behavior should not require a Rust developer or a Rust rebuild merely because HTML, CSS, SCSS, or JavaScript changed.

### Progressive interaction model

The project should use the least complicated mechanism that fits the interaction:

```text
ordinary HTML/CSS
    ↓
Bootstrap or ordinary JavaScript
    ↓
HTML-first server interaction such as htmx
    ↓
native Topcoat component/shard/procedure
    ↓
strongly typed reactive behavior where the extra machinery is justified
```

This is deliberately not an instruction to replace Topcoat with htmx. Native Topcoat remains valuable for interactions that benefit from its typed server/browser coordination, procedures, signals, shards, or other framework-specific capabilities.

### Runtime view renderer

MiniJinja 2.24.0 is the View/DX Architecture v1 runtime renderer.

The decision is based on the completed renderer investigation and the first real application slice: external HTML renders through Topcoat, ordinary HTML edits can be observed without a Rust rebuild, typed Rust view models remain on the server side, HTML values are escaped by default, and renderer failures remain ordinary application errors rather than Rust compiler failures.

The application accesses MiniJinja through the narrow project-owned `ViewRenderer` boundary rather than spreading renderer-specific APIs throughout web code. Tera also passed the earlier feasibility experiments and remains useful comparison evidence, but it is not the v1 renderer.

### View security boundary

Runtime views must preserve the same security posture as the compiled Topcoat view path.

Required invariants:

- untrusted values are escaped by default
- raw HTML is never trusted merely because it came through a view model
- the runtime renderer owns the narrow trusted boundary used to pass fully rendered HTML into Topcoat
- arbitrary request-controlled view paths are forbidden
- view lookup must prevent path traversal and arbitrary file access
- development reload behavior must not weaken production view integrity; only the explicit `development` environment enables filesystem autoreload
- browser action identifiers and values remain untrusted input
- authorization, validation, and business rules remain server-authoritative
- production uses a fixed application-owned view root, disables rich template debug formatting, and validates registered views at startup; deployment packaging/read-only filesystem policy remains an operational concern

### Rusty responsibility

Rusty should automate project conventions, not invent another frontend framework.

Rusty now provides `make:view` as the foundational View/DX scaffolding primitive. It creates an external HTML page template under `resources/views/pages`, registers an app-owned `ViewName`, adds that view to startup validation, refuses existing paths or symbols, and supports a no-write `--dry-run` preview. It does not infer routes, authentication, controller behavior, htmx behavior, or persistence wiring.

`make:controller` now scaffolds the corresponding web presentation-controller module under `src/web/controllers`, registers it with the existing `pub(in crate::web)` visibility convention, refuses path-like or colliding names, and supports the same no-write `--dry-run` workflow. It creates only the presentation-controller boundary; controller methods and application behavior remain explicit developer decisions.

`make:view-model` now scaffolds a serializable presentation model under `src/web/view_models`, registers its private module, re-exports the generated `<Name>View` through the existing `pub(in crate::web)` boundary, refuses path-like or colliding names, and supports `--dry-run`. The generated type is intentionally minimal; fields and application-specific data remain explicit developer decisions.

`make:page` now scaffolds a complete public HTML-first GET page as one atomic plan: page module, presentation controller, asset-aware serializable view model, registered external HTML view, and all required module/view registrations. It preflights the entire scaffold before writing, refuses existing routes or boundaries, validates only static GET paths in this first version, supports `--dry-run`, and rolls back applied changes if a later write fails. It deliberately does not infer authentication, database access, htmx behavior, forms, mutations, or application use cases.

Likely future View/DX scaffolding includes commands equivalent to:

```text
rusty make:component ...
rusty make:resource ...
```

Exact command names are not yet committed.

Rusty may create route/controller/view registrations and validation checks, but it should not become a custom HTML parser, template language, browser-reactivity VM, or Livewire clone.

## Frontend asset source layout

Authored browser-facing source lives under `resources/`, while generated browser artifacts remain under `generated/`.

```text
resources/
├── views/
├── css/
├── scss/
├── js/
└── images/

generated/
├── app.css
├── app.css.map
├── bootstrap.bundle.min.js
└── htmx.min.js
```

Plain CSS and SCSS are both first-class styling options. Bootstrap is consumed from pinned source SCSS, while htmx and the Bootstrap JavaScript bundle are copied from pinned development dependencies. `generated/` is build output rather than authored application source.

## Rusty CLI developer tooling

Rusty CLI is a separate Rust crate under:

```text
tools/rusty-cli/
```

It has its own `Cargo.toml` and `Cargo.lock`, keeping developer-tool dependencies out of the production web application's dependency graph.

The containerized executable is accessed through the location-independent repository-root wrapper:

```text
<repository-root>/rusty
```

When the optional Bash alias is installed, commands can be invoked simply as `rusty ...` from any directory.

Current commands include:

```text
rusty doctor
rusty audit

rusty migration <Name>
rusty make:migration <Name>
rusty migration <Name> --simple

rusty make:test <module-path> <test-name>
rusty test <module-path> <test-name>

rusty make:command <command-name>
rusty make:command <command-name> --dry-run
rusty make:view <page/path>
rusty make:view <page/path> --dry-run
rusty make:controller <Name>
rusty make:controller <Name> --dry-run
rusty make:view-model <Name>
rusty make:view-model <Name> --dry-run
rusty make:page <Name> --path /route
rusty make:page <Name> --path /route --dry-run
```

Current responsibilities include:

- project-root discovery
- environment diagnostics
- SQLx-compatible migration generation
- reversible migration pairs by default
- optional simple/irreversible migrations
- safe refusal to overwrite existing migrations
- snake_case normalization
- host UID/GID-safe file generation
- module-aware test scaffolding
- automatic `tests/mod.rs` maintenance
- automatic parent `#[cfg(test)] mod tests;` registration
- path traversal and invalid Rust identifier rejection
- safe refusal to overwrite existing test files
- deterministic scaffolding of new Rusty CLI commands
- dry-run command-generation previews
- deterministic generator markers for Rusty's command enum, module registry, and dispatcher
- project-local Rust security/audit rules for public input, validation/UUID parsing, credential verification, dynamic SQL, shell execution, and persistence sink flows

The default reversible migration convention is:

```text
<version>_<description>.up.sql
<version>_<description>.down.sql
```

Module-local tests follow the application's actual module organization. For example:

```text
src/db/models/contact/
├── mod.rs
├── model.rs
├── persistence.rs
└── tests/
    ├── mod.rs
    └── persistence.rs
```

A command such as:

```bash
rusty make:test db/models/contact validation
```

creates:

```text
src/db/models/contact/tests/validation.rs
```

and updates module registration as needed.

`rusty make:command` is the Rust equivalent of an Artisan-style command scaffolder. It creates a new command source module and registers the corresponding CLI enum variant and dispatcher entry. `--dry-run` should be used when previewing a new command name or generator mapping.

### Planned Rusty commands

Future Rusty work should only be added where it materially improves project consistency or development velocity.

The next planned areas are:

- View/DX scaffolding after the renderer and interaction conventions are proven through real application slices
- PostgreSQL-backed model generation/synchronization using real database metadata
- additional consistency/diagnostic commands only where they provide clear value

Database-backed model generation should target the canonical module structure, use actual primary-key metadata, map known PostgreSQL types explicitly, and fail rather than silently guessing unknown database types.

The legacy PHP Rusty CLI is not part of this template.

## Secrets and configuration

The preferred secrets mechanism is **Docker Compose secrets**, not `.env`.

Local development secrets live under:

```text
.secrets/
```

This directory is ignored by Git.

The current setup uses four independently generated password files:

```text
.secrets/postgres_bootstrap_password
.secrets/postgres_migrator_password
.secrets/postgres_app_password
.secrets/postgres_test_password
```

For local file-backed Compose secrets, keep `.secrets/` at `0700` and the contained password files at `0644`. The directory permission protects the host-side files, while the file permissions allow non-root service users such as PostgreSQL's `postgres` user to read explicitly mounted Compose secrets.

Because the repository itself is bind-mounted into development containers, `.secrets/` is masked inside those containers with `tmpfs`. A service therefore sees only the secrets explicitly granted under `/run/secrets/`.

The intended privilege model is:

```text
postgres
    bootstrap superuser
    normal PostgreSQL initialization / administration only

topcoat_migrator
    non-superuser schema owner
    migrations and schema evolution only

topcoat_app
    non-superuser runtime role
    SELECT / INSERT / UPDATE / DELETE only

topcoat_test_admin
    isolated test PostgreSQL administrator
    per-test database creation/destruction only
    never used by the normal application database
```

The Topcoat application receives only:

```text
/run/secrets/postgres_app_password
```

The migration service receives only:

```text
/run/secrets/postgres_migrator_password
```

The test runner receives only:

```text
/run/secrets/postgres_test_password
```

The normal application and migration stack do not receive the test credential, and the test runner explicitly refuses to start if normal bootstrap, migrator, or application secrets are visible.

Non-secret database configuration such as host, port, database name, and username may use ordinary container environment variables.

The normal Rust application reads its database password directly from the mounted secret file instead of converting it into a persistent password-bearing `DATABASE_URL` environment variable.

The test runner constructs `DATABASE_URL` only inside the disposable test process because SQLx's database-test machinery requires a runtime database URL for per-test database management.

`.env` may be supported later as an optional convenience fallback for template users, but it is not the preferred configuration mechanism.

## Security philosophy

Rust memory safety does not replace application security.

The target is a security posture comparable to or stronger than a mature Laravel application.

Security controls are evaluated independently, including:

- authentication
- authorization
- session security
- CSRF protection
- origin validation
- input validation
- output escaping
- SQL injection prevention
- XSS prevention
- rich HTML sanitization
- rate limiting
- secure cookies
- security headers
- CSP
- file-upload handling
- secret management
- least-privilege database roles
- audit logging
- dependency auditing

Missing Topcoat functionality must be implemented using mature, well-reviewed Rust crates or proven industry patterns rather than ad-hoc replacements.

## Authentication and session baseline

The current authentication architecture deliberately separates application credential policy from the underlying Topcoat session transport.

Current invariants:

- authentication provider selection happens at startup
- the provider is not selectable from request input
- unsupported providers fail closed
- there is no fallback/downgrade path
- the current provider is `local-topcoat`
- application code owns credential verification and security policy
- Topcoat provides the server-side session infrastructure
- Argon2id is used for password hashing
- login throttling occurs before expensive password verification
- unknown-user requests still perform password-verification work
- invalid credentials return generic browser-facing failures
- persisted server-side session state is authoritative
- protected endpoints authenticate independently
- logout revokes access
- session/backend failures are not intentionally collapsed into false unauthenticated responses
- passwords are opaque inputs and are not generically trimmed or normalized
- email lookup is normalized separately from password handling
- passwords, raw session tokens, and token hashes must not be logged

The current local session profile uses bounded idle and absolute lifetimes plus a refresh window. Session rotation remains an explicit security seam rather than being simulated with custom application tokens.

The application also provides an operator-controlled user-provisioning flow that requires an explicit `viewer` or `operator` role while continuing to read the password interactively instead of accepting it through command-line arguments or environment variables.

Public self-registration, password reset, email verification, and MFA remain future account-lifecycle capabilities rather than requirements for the completed POC authentication baseline.

## User-input trust model

All client input is untrusted.

The intended request path is:

```text
HTTP input
    ↓
typed input struct
    ↓
field-specific normalization
    ↓
validation
    ↓
validated input
    ↓
authorization / business rules
    ↓
persistence
```

Validation and sanitization are not treated as the same operation.

Normal text should be validated and rendered through Topcoat's normal escaped output mechanisms.

Raw HTML is never trusted merely because it passed validation. If rich HTML support is introduced, it will use an allowlist sanitizer and a dedicated trusted HTML type before unescaped rendering is permitted.

Database queries must use SQLx parameter binding. User-controlled values must never be interpolated directly into SQL strings.

## Database architecture

PostgreSQL is the database target.

SQLx is the intended Rust database toolkit rather than a traditional ORM.

The design favors:

- explicit SQL
- compile-time checked queries
- strongly typed row models
- connection pooling
- embedded/versioned migrations
- compile-time model/query contracts
- checked-in SQLx offline metadata
- restricted locations for raw SQL
- separation of migration authority from runtime DML authority
- isolated database-backed persistence tests

Normal model-specific SQL lives with that model's persistence implementation.

Current structure:

```text
src/db/
├── models/
│   └── contact/
│       ├── mod.rs
│       ├── model.rs
│       ├── persistence.rs
│       └── tests/
│           ├── mod.rs
│           └── persistence.rs
│
└── queries/
```

Responsibilities:

- `model.rs` — persisted row/data type
- `persistence.rs` — SQL and persistence behavior specific to that model
- `tests/` — module-local persistence and behavior tests
- `queries/` — complex joins, projections, reports, or queries that do not naturally belong to one model

Raw application SQL should not appear in page handlers, components, or unrelated services.

Migrations remain under:

```text
migrations/
```

because migration SQL defines the database schema itself.

Reversible migrations use SQLx's native paired naming convention:

```text
<version>_<description>.up.sql
<version>_<description>.down.sql
```

Generate them through Rusty CLI instead of creating files manually:

```bash
rusty migration CreateContacts
```

The Compose migration service runs under `topcoat_migrator` and completes before the runtime application starts. The `topcoat_app` runtime role does not receive schema-owner authority.

The first migration creates `public.contacts` with UUID primary keys, bounded non-empty names, bounded email storage, timestamps, and a case-insensitive unique index on `lower(email)`. UUID values are generated in Rust rather than by the database.

The current Contact persistence API includes create, read, update, delete, and list/count operations. Production web mutations route through the application layer rather than invoking persistence directly, and list rendering consumes an application query/read model rather than a persistence row structure.

## Migration build tracking

The repository-root `build.rs` exists specifically for SQLx embedded migration tracking:

```rust
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
```

The repository-root `.gitattributes` pins SQL migration files to LF line endings:

```gitattributes
*.sql text eol=lf
```

This keeps SQLx migration hashes stable across development platforms.

## Model/schema contracts

A primary design requirement is that Rust database structs remain synchronized with the actual PostgreSQL schema.

The current Contact implementation uses complementary SQLx compile-time contracts.

### Row-shape contract

A `query_as!()` contract selects the complete `public.contacts` row into `Contact`.

This verifies the expected field set and catches added, removed, renamed, or SQL-type-incompatible model fields.

The contract was deliberately tested by temporarily adding an imaginary field to `Contact`; compilation failed because SQLx could not initialize that field from the query result.

### Nullability contract

`query_as!()` alone is intentionally not treated as a strict nullability proof because SQLx permits some compatible widening such as assigning a non-null SQL field into `Option<T>`.

A second `query!()` contract therefore derives PostgreSQL's concrete field types and assigns those values directly into a `Contact` initializer.

This makes Rust's assignment rules enforce the exact expected nullability.

The contract was deliberately tested by temporarily changing the non-null `email: String` field to `email: Option<String>`; compilation failed with the expected type mismatch.

### SQLx offline metadata

Compile-time SQLx query metadata is checked into:

```text
.sqlx/
```

This allows ordinary builds to validate query macros without requiring a live development database.

Refresh metadata against the real schema with a temporary runtime `DATABASE_URL`:

```bash
docker compose stop app

docker compose run --rm --no-deps \
  -e HOST_UID="$(id -u)" \
  -e HOST_GID="$(id -g)" \
  app \
  sh -c '
    password="$(cat "$DB_PASSWORD_FILE")"

    export DATABASE_URL="postgres://${DB_USER}:${password}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

    cargo sqlx prepare -- --all-targets

    chown -R "${HOST_UID}:${HOST_GID}" /app/.sqlx
  '

docker compose up -d app
```

Verify that checked-in metadata still matches the database:

```bash
docker compose stop app

docker compose run --rm --no-deps app sh -c '
  password="$(cat "$DB_PASSWORD_FILE")"
  export DATABASE_URL="postgres://${DB_USER}:${password}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
  cargo sqlx prepare --check -- --all-targets
'

docker compose up -d app
```

Use `sh -c`, not a login shell, for these one-shot container commands so the container's Cargo path is preserved.

Custom reflection or procedural macros should only be introduced if SQLx does not provide sufficient guarantees.

## Forms and persisted models

Incoming request structures and persisted database structures are separate concepts.

The established Contact mutation flow is:

```text
untrusted browser/procedure input
        ↓
typed create/update command
        ↓
field-specific normalization
        ↓
validation
        ↓
validated domain input
        ↓
application boundary
        ↓
authorization / business rules
        ↓
Contact persistence
        ↓
Contact / application result
```

The application does not deserialize arbitrary HTTP input directly into persisted database models.

This provides a strongly typed boundary against accidental privilege fields, mass-assignment-style mistakes, and unvalidated data reaching persistence code.

Contact create, update, and delete operations now route through the application layer. Contact listing is also isolated behind an application query/read-model boundary so web presentation code does not consume persistence structures directly.

Passwords or similarly opaque secrets must not be subjected to generic trimming or normalization. Normalization is field-specific.

## Testing architecture

Testing follows a two-tier model.

Pure logic should use ordinary Rust unit tests whenever no database is required.

Database-backed persistence behavior uses `#[sqlx::test]` against the isolated test PostgreSQL server.

The canonical complete-suite command is:

```bash
./test
```

The repository-root `test` wrapper runs the one-shot Compose `test` service. Its safety script verifies the complete expected testing contract before Cargo executes.

Required test invariants include:

```text
APP_ENV=testing
DB_HOST=postgres-test
DB_PORT=5432
DB_NAME=postgres
DB_USER=topcoat_test_admin
DB_PASSWORD_FILE=/run/secrets/postgres_test_password
```

The safety gate also verifies that normal bootstrap, migrator, and application database secrets are not mounted or exposed through the repository bind mount.

`postgres-test` is a separate PostgreSQL server attached to the dedicated test network. It does not use the normal development data volume and stores its cluster in `tmpfs`.

SQLx creates a fresh logical database for each database-backed `#[sqlx::test]`, applies the repository migrations, and allows the individual tests to run independently.

Current Contact persistence coverage verifies:

- lookup of a missing Contact returns `None`
- lookup of a persisted Contact returns the expected row
- `count()` returns the correct number of Contacts
- `find_all()` returns Contacts in deterministic descending creation order

All four persistence tests are currently green.

The warm PostgreSQL service and dedicated `/cargo-test-target` build cache keep the local feedback loop fast. A verified warm complete-suite run finished in approximately 1.3 seconds.

Rusty has a complementary but separate testing responsibility:

```bash
rusty make:test db/models/contact validation
```

creates a test file and module registration. It does **not** execute the test suite.

## Development

Build and start the normal stack:

```bash
docker compose up --build
```

Or start existing images:

```bash
docker compose up -d
```

Check normal service state:

```bash
docker compose ps
```

Follow application logs:

```bash
docker compose logs -f app
```

Run the canonical container build check:

```bash
docker compose exec app cargo check --locked
```

Run the host build check:

```bash
cargo check --locked
```

Formatting:

```bash
cargo fmt --check
```

Linting:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Run the complete safe project test suite:

```bash
./test
```

Inspect all containers, including the warm test database:

```bash
docker compose ps -a
```

Rusty CLI checks:

```bash
cargo fmt --manifest-path tools/rusty-cli/Cargo.toml --check
cargo check --manifest-path tools/rusty-cli/Cargo.toml --locked
```

Rusty CLI usage:

```bash
rusty --help
rusty doctor
rusty audit
rusty migration CreateContacts
rusty make:test db/models/contact validation
rusty make:command model:sync --dry-run
```

The repository-root `./rusty` wrapper remains usable even when the global Bash alias is not configured.

## Topcoat development reload

Topcoat 0.9's development loop has been inspected directly as part of the View/DX work.

Verified behavior:

- the previous successful application can keep serving while a rebuild is pending
- a failed rebuild does not require the previous working application to be torn down
- the dev broadcast process survives application restarts
- the browser protocol distinguishes rebuilding, build failure, application exit, reload, and already-up-to-date states
- successful reloads can refresh the current document through Topcoat's browser-side morph/update behavior rather than relying only on a blind full-page reload
- Topcoat watches local workspace sources broadly enough that future runtime view directories may also trigger its watcher, even when Cargo ultimately has no Rust binary change to rebuild

`topcoat::dev::script()` is currently retained because it provides Topcoat's browser-side development reload/status behavior.

Under the current Docker configuration, its dynamically selected development endpoint is not exposed correctly to the host browser and can produce a refused `dev.js` request.

This does not affect application functionality. The development endpoint should be corrected separately rather than removing the feature.

The HTML-first view system and Topcoat development loop remain deliberately separate: MiniJinja autoreload handles ordinary HTML view edits, while Topcoat's development machinery remains responsible for Rust rebuild/reload behavior. The project does not reimplement Topcoat's private CLI reload protocol.

## Repository policy

Commit:

- Rust source
- frontend source
- migrations
- `.sqlx/` offline query metadata
- `Cargo.lock`
- `package-lock.json`
- `rust-toolchain.toml`
- Docker configuration
- `docker/test/run.sh`
- repository-root `test` wrapper
- `build.rs`
- `.gitattributes`
- `tools/rusty-cli/`
- Rusty CLI's `Cargo.lock`
- repository-root `rusty` wrapper

Do not commit:

- `.secrets/`
- `.env`
- `target/`
- `tools/rusty-cli/target/`
- `node_modules/`
- generated frontend assets
- database data volumes
- private keys or credentials

## POC roadmap

### POC 0 — Frontend/runtime baseline

**Status: Green**

- SSR
- routing/layouts
- Bootstrap
- SCSS
- plain CSS
- assets
- client reactivity
- Dockerized development

### POC 1 — Application/data foundation

**Status: Green baseline**

Implemented:

- modular application structure
- PostgreSQL
- SQLx
- connection pool
- typed configuration
- Docker secrets
- least-privilege PostgreSQL roles
- dedicated migration runner
- Rusty CLI migration scaffolding
- embedded/versioned migrations
- LF-stable migration hashing
- `contacts` migration
- Contact persisted row model
- SQLx offline metadata
- compile-time row-shape contract
- compile-time nullability contract
- Contact create/read/update/delete persistence
- isolated PostgreSQL test environment
- fail-closed test database safety gate
- per-test SQLx databases
- module-local persistence tests
- fast warm test feedback loop
- application-layer Contact mutations
- application Contact list query/read-model boundary

### Developer tooling — Rusty CLI

**Status: Green baseline**

Implemented:

- standalone Rust crate
- independent Cargo dependency graph
- Dockerized developer-tool service
- location-independent root wrapper
- host UID/GID-safe generated files
- optional global Bash alias
- `doctor`
- `audit`
- reversible SQLx migration generator
- simple/irreversible migration option
- migration compatibility alias
- module-aware `make:test`
- automatic test module registration
- safe test scaffold overwrite refusal
- `make:command`
- deterministic Rusty command-registration markers
- command scaffold `--dry-run`
- public-input/persistence/security audit rules

Planned:

- View/DX scaffolding after the architecture is finalized
- PostgreSQL-backed model generation and synchronization
- additional consistency/diagnostic commands only where they provide clear value

### POC 2 — Validation and forms

**Status: Green baseline**

Implemented:

- typed Contact request/command inputs
- field-specific normalization
- validation
- UUID parsing
- application mutation boundary
- browser-facing mutation states
- create/update/delete Contact workflow

Future work can extend these patterns to additional resource types rather than re-proving the same baseline.

### POC 3 — Authentication and sessions

**Status: Green baseline for current POC scope**

Implemented:

- startup-selected authentication provider
- no request-controlled provider switching
- no fallback/downgrade path
- Argon2id password hashing and verification
- generic invalid-login behavior
- login throttling
- unknown-user verification work
- persisted Topcoat-backed sessions
- login
- logout
- protected routes
- authenticated navigation
- session persistence
- bounded idle/absolute/refresh session profile
- operator user creation
- authentication/session lifecycle tests

Future account-lifecycle capabilities:

- public registration, if required by an application
- password reset
- email verification
- MFA

### Cross-cutting — View/DX architecture

**Status: Feasibility proven; v1 design in progress**

Verified experimentally:

- runtime external HTML views can coexist with Topcoat 0.9
- HTML/view edits do not inherently require Rust recompilation
- runtime view syntax failures can remain HTTP/runtime failures rather than compiler failures
- MiniJinja and Tera both satisfy the basic runtime-view requirement
- MiniJinja currently has the stronger development-DX signal
- Topcoat's official htmx integration works for server-owned POST updates
- native Topcoat procedure/signal wiring and HTML-first interactions can coexist

Current design direction:

- HTML-first default for ordinary presentation
- separate JS/SCSS/CSS resources
- native Topcoat reactivity as an opt-in capability
- Rust controllers/application logic feeding typed view models
- Rusty scaffolding conventions instead of a custom view/reactivity framework
- explicit dev/prod view-loading and security policy

### POC 4 — Authorization

**Status: Planned**

- policies
- RBAC
- ownership rules
- route/action authorization
- negative authorization tests

### POC 5 — Web security hardening

**Status: Partially implemented; broader hardening planned**

Already represented in the current baseline:

- authentication throttling
- secure server-side session handling
- generic credential failures
- least-privilege database roles
- typed/validated application input
- default output escaping
- Rusty security auditing

Still to evaluate or complete as application-wide policy:

- CSRF/origin protections for all relevant mutation paths
- CSP
- security headers
- request-size limits
- file-upload policy
- broader rate-limiting policy
- production error handling
- application audit logging

### POC 6 — Production readiness

**Status: Planned**

- complete test matrix
- dependency auditing
- container hardening
- observability
- deployment strategy
- production reverse proxy/TLS architecture
- production runtime-view validation/preload/package policy

## Decision criteria

The final decision will not be based solely on performance or preference for Rust.

Topcoat/Rust must demonstrate that it can provide:

- maintainable application organization
- secure defaults or explicit secure replacements
- reasonable development velocity
- predictable framework behavior
- strong testing ergonomics
- manageable dependency risk
- clean deployment and operations
- ordinary HTML/CSS/JavaScript work that does not unnecessarily require Rust recompilation
- a progressive path from standard web development to native Topcoat reactivity
- clear separation between routes/controllers, application logic, views, JavaScript, and styles

If recreating Laravel's mature safeguards or developer ergonomics creates excessive custom security or maintenance burden, that will weigh against a rewrite even if the Rust implementation performs well.
