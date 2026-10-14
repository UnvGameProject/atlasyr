# Atlasyr

**Rust at the core. Web without the detour.**

Atlasyr is a production-oriented web application framework and platform for **Rust-centric systems**, built on [Topcoat](https://github.com/tokio-rs/topcoat).

Its purpose is not to replace Laravel, Rails, Django, or other mature general-purpose web frameworks. Atlasyr exists for a different center of gravity:

> **Use Atlasyr when the important part of your application is already Rust.**

That includes systems where Rust is carrying domain logic, concurrency, low-latency services, computational workloads, financial infrastructure, blockchain or crypto infrastructure, realtime workloads, games, simulation, graphics, distributed systems, streaming data, or other capabilities that would be awkward to place behind a separate web stack written in another backend language.

Atlasyr keeps that Rust core intact while making the web-facing application layer secure, maintainable, productive, and accessible to ordinary frontend development.

## Project status

**Qualification complete. Product foundation begins here.**

The original Topcoat proof of concept completed its production-readiness qualification through POC 6 and reached an **Adopt With Constraints** decision for Rust-heavy application contexts.

The frozen qualification checkpoint is:

- branch baseline: `dx/contacts-architecture`
- final qualification commit: `988ed1d`
- release-gate tag: `production-release-gate-v1`
- POC 6 closeout tag: `poc-6-production-readiness-v1`
- pinned framework baseline: Topcoat `0.10.0`
- Rust toolchain: `1.98.1`
- application release gate: `./release-gate`

The repository name and some historical terminology still reflect the qualification period. Product-transition work will rename and reorganize those surfaces deliberately rather than rewriting Git history.

The post-qualification product roadmap lives in [`ROADMAP.md`](ROADMAP.md).

## Built on Topcoat

Atlasyr exists because Topcoat already provides substantial Rust web-framework machinery: routing, request context, sessions, cookies, assets, server rendering, client reactivity, procedures, shards, navigation, mail, and integrations such as htmx.

Atlasyr is an **independent project built on Topcoat**. It is not an official Tokio or Topcoat project, and it must not obscure the work performed upstream.

Topcoat receives explicit architectural credit because Atlasyr is not intended to fork or rebrand Topcoat's core framework work. Atlasyr adds a production application architecture, security conventions, HTML-first presentation option, developer tooling, deployment/release contracts, and Rust-heavy systems integrations around the upstream framework.

The responsibility boundary is intentionally explicit:

```text
Topcoat
    routing / request context / sessions / cookies
    assets / rendering / runtime / procedures / shards
    upstream browser integration and framework primitives
        |
        v
Atlasyr
    production application conventions
    security and authorization policy
    HTML-first presentation architecture
    application/domain/persistence boundaries
    Rusty developer tooling
    release / deployment contracts
    provider and systems integrations
        |
        v
Application
    domain-specific behavior
    business rules
    product UI
    infrastructure choices
```

Generic improvements that belong naturally in Topcoat should be considered for upstream contribution rather than permanently duplicated inside Atlasyr.

## Why Atlasyr exists

A Rust-heavy team should not have to introduce an unrelated backend language merely because the system needs a browser interface.

The problem Atlasyr is trying to solve is the architectural detour that often appears after a Rust backend becomes important enough to need a serious web surface:

```text
Rust domain / services / compute
        |
        | translation boundary
        v
different backend framework / language
        |
        v
web application
```

Atlasyr instead aims for:

```text
Rust domain / services / compute
        |
        v
Atlasyr application boundaries
        |
        +--> ordinary HTML / CSS / JavaScript / htmx
        |
        +--> native Topcoat reactivity when justified
```

The value is not merely faster HTTP. The stronger value is **continuity**:

- Rust domain types can remain Rust domain types.
- Tokio-based concurrency does not have to be translated into a second worker/runtime philosophy.
- Rust libraries and internal crates can remain directly usable by the web application.
- strongly typed authorization, validation, persistence, and application contracts can remain end to end.
- high-performance compute and streaming capabilities can sit directly behind the web-facing application layer.
- frontend specialists can still work in ordinary HTML, CSS, SCSS, JavaScript, and htmx without becoming advanced Rust developers.

## Who Atlasyr is for

Atlasyr is aimed primarily at:

- Rust-first engineering teams that need a production web application around an existing Rust core;
- fintech, trading, payments, and low-latency systems;
- blockchain, crypto, and distributed-ledger applications;
- realtime and message-driven systems;
- systems using Tokio heavily for concurrency and asynchronous I/O;
- game, simulation, geometry, media, or graphics workloads;
- data-processing and streaming systems;
- AI/ML infrastructure whose control plane or product surface needs a web layer;
- globally distributed or multi-region systems;
- applications that need to expose computational or infrastructure-heavy Rust capabilities safely to browsers or operators.

Atlasyr can also support ordinary CRUD and business applications, but ordinary CRUD alone is not the reason for the framework to exist.

## Who Atlasyr is not trying to replace

If an application is primarily a conventional website, storefront, CMS, small-business application, or ordinary database-backed product with no meaningful Rust center of gravity, a mature general-purpose framework such as Laravel may still be the better choice.

Laravel + Livewire remains an important quality reference for Atlasyr because it demonstrates what a mature framework can make safe, predictable, and fast:

- authentication and authorization;
- validation and forms;
- sessions and browser security;
- scaffolding and developer tooling;
- testing;
- background work;
- deployment conventions;
- common web application ergonomics.

Atlasyr uses that maturity as a **bar**, not as a checklist to clone.

The goal is not:

> "Laravel, but in Rust."

The goal is:

> "A web framework that lets a Rust-centric system gain a serious web application without giving up the reasons the system chose Rust."

## Product principles

### 1. Rust-centric, not Rust-everywhere

Rust should own the parts where Rust adds value. Ordinary markup and styling should remain ordinary markup and styling.

HTML, CSS, SCSS, JavaScript, browser assets, and htmx interactions must remain approachable to frontend specialists who do not need to understand the application's deeper Rust internals.

### 2. HTML-first, not HTML-only

Atlasyr uses the least complicated interaction model that fits the job:

```text
ordinary HTML / CSS
        |
        v
Bootstrap or ordinary JavaScript
        |
        v
HTML-first server interaction such as htmx
        |
        v
native Topcoat component / shard / procedure
        |
        v
strongly typed reactive behavior when the extra machinery is justified
```

Native Topcoat remains a first-class capability. HTML-first presentation prevents the framework from forcing every ordinary page into Rust-authored reactive machinery.

### 3. Core simplification before code generation

Developer velocity must improve in two places:

1. simplify repetitive framework APIs and application wiring;
2. use Rusty to automate the mechanical work that still remains.

Generating ten repetitive steps is useful. Designing the framework so only three steps are necessary is better.

### 4. Security is an application property

Rust memory safety is valuable, but it does not replace authentication, authorization, input validation, output escaping, secure sessions, origin policy, rate limiting, dependency auditing, secret handling, upload policy, audit history, production error handling, or deployment security.

Atlasyr keeps those concerns explicit and testable.

### 5. Upstream before duplication

Topcoat is a moving target and remains pre-1.0. Atlasyr must expect upstream capabilities and APIs to change.

Before Atlasyr builds or expands a cross-cutting capability, the project must ask:

- Does Topcoat already provide it?
- Is Topcoat actively building it?
- Can Atlasyr use the upstream capability directly?
- Is a narrow Atlasyr adapter sufficient?
- Does Atlasyr have a genuinely different Rust-heavy use case that warrants an alternative?
- If Atlasyr already owns an implementation, should it be migrated, retained as a selectable provider, or deprecated?

### 6. Do not wrap Topcoat merely to hide Topcoat

Atlasyr should not create a second framework API around every upstream API.

A project-owned seam is justified when it provides one or more of:

- stable application policy;
- meaningful provider choice;
- a security boundary;
- a compatibility boundary around proven upstream churn;
- a domain/application abstraction that should not depend on web-framework types.

Where direct Topcoat usage is clear and stable, direct Topcoat usage is preferred.

### 7. Mature infrastructure stays infrastructure

Atlasyr should integrate with PostgreSQL, MongoDB, Redis, RabbitMQ, NATS, Kafka, object stores, tracing systems, and other mature infrastructure rather than reimplementing them.

For example, Atlasyr may make MongoDB topology-aware access or shard/tenant routing easier, but MongoDB remains responsible for database sharding.

### 8. Advanced Rust capabilities should remain composable

A Rust pricing engine, blockchain client, simulation core, geometry library, Tokio service, or streaming engine should be able to plug into the application without being translated into another backend framework's internal model.

### 9. Production behavior is part of the framework experience

A web framework is not complete merely because it serves a page.

Atlasyr treats dependency integrity, build reproducibility, production packaging, least privilege, health semantics, graceful shutdown, proxy/TLS contracts, telemetry, and release gates as product concerns.

### 10. Evidence before abstraction

New framework abstractions should come from repeated application pressure, not speculative architecture.

The first post-qualification product milestone therefore measures a second real feature before Atlasyr expands Rusty or introduces broad new core abstractions.

## Topcoat compatibility doctrine

Topcoat compatibility is a permanent product function, not an occasional upgrade chore.

Each Atlasyr capability should be classified in an upstream capability ledger as one of:

| State | Meaning |
| --- | --- |
| **Topcoat-native** | Atlasyr uses the upstream feature directly. |
| **Atlasyr adapter** | Atlasyr adds policy or a narrow compatibility seam around an upstream primitive. |
| **Atlasyr-owned** | Atlasyr owns the capability because upstream does not provide it or the responsibility belongs at the application/platform layer. |
| **Selectable provider** | Multiple implementations solve meaningfully different use cases and the application may choose. |
| **Experimental** | The capability is being evaluated and is not yet part of the stable framework contract. |
| **Deprecated / upstream-replaced** | Upstream now provides the preferred implementation; Atlasyr supplies a migration path and removes duplication deliberately. |

For every Topcoat release considered for adoption, Atlasyr should review:

1. release notes and breaking changes;
2. Topcoat CLI/runtime version coupling;
3. Rust/MSRV/toolchain changes;
4. APIs Atlasyr consumes;
5. security-sensitive behavior;
6. sessions/cookies/request-context behavior;
7. routing and middleware behavior;
8. assets and runtime behavior;
9. htmx and other browser integrations;
10. production packaging/deployment implications;
11. overlap with Atlasyr-owned capabilities;
12. opportunities to delete or simplify Atlasyr code.

Stable Atlasyr releases should pin a reviewed Topcoat version while Topcoat remains pre-1.0. Atlasyr's own versioning should remain independent, with an explicit compatibility matrix documenting which Topcoat versions each Atlasyr release has actually qualified.

A non-release canary against Topcoat `main` is planned so likely breakage can be discovered before an upstream release rather than during an emergency upgrade.

## Development velocity objective

The qualification exercise rated current Atlasyr-style development velocity below the Laravel reference because significant framework plumbing had to be created during the POC.

That score is **not** treated as permanent.

The product target is to raise velocity for the intended Rust-heavy user without trading away type safety, explicit security boundaries, or maintainability.

Work will focus on:

- reducing repeated core wiring;
- improving framework-owned adapters and error conversions;
- making common application boundaries predictable;
- expanding Rusty only where automation removes real repeated work;
- measuring the before/after workflow on representative application features;
- keeping ordinary frontend iteration independent from Rust rebuilds.

The first major product milestone after upstream compatibility work is **Development Velocity v1**.

## Roadmap

The complete roadmap is maintained in [`ROADMAP.md`](ROADMAP.md).

The sequence begins with:

1. qualification closeout and product transition;
2. permanent Topcoat upstream-sync / compatibility infrastructure;
3. Development Velocity v1 using a representative second application feature;
4. framework-core stabilization;
5. secure web-application essentials;
6. data/storage integrations;
7. async, messaging, and realtime systems;
8. observability and reliability;
9. security and identity expansion;
10. global/distributed-system capabilities;
11. optional advanced compute, WASM, and WebGPU integration;
12. provider ecosystem, distribution, documentation, and long-term maturity.

Advanced capabilities are intentionally later than compatibility and developer-velocity work. Atlasyr should first become easy to evolve and easy to build with.

## Working name

**Atlasyr** is the current working product identity.

The name preserves the "Atlas" idea of a load-bearing platform while avoiding the generic `Atlas-R` form. It is intended to sound like a distinct framework rather than an edition of Topcoat.

This is a product-development name, not a claim of trademark, domain, package, or organization-name availability. Formal naming clearance is part of the product-transition roadmap before a public release.


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

# Atlasyr Rusty CLI
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
cargo fmt --all -- --check
```

Run Clippy diagnostics:

```bash
cargo clippy --workspace --all-targets --all-features --locked
```

The repository currently carries a small, explicitly tracked legacy Clippy warning baseline. The canonical `./release-gate` de-duplicates repeated all-target emissions and fails if that known warning set changes. Project-wide `-D warnings` is therefore not claimed as green until that legacy baseline is resolved.

Run the same checks in Docker when needed:

```bash
docker compose exec app cargo fmt --all -- --check
docker compose exec app cargo clippy --workspace --all-targets --all-features --locked
```

### Testing

The canonical application test entrypoint is:

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

`cargo test` remains the underlying Rust test runner. Rusty does not execute application tests; `rusty make:test` only scaffolds test files. The standalone Rusty CLI crate has its own regression suite:

```bash
cargo test --manifest-path tools/rusty-cli/Cargo.toml --locked
```

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

### Production release gate

The canonical pre-release validation entrypoint is:

```bash
./release-gate
```

`./release-gate` composes the production-readiness matrix rather than duplicating those checks in CI. It verifies formatting, locked all-target compilation, the exact tracked Clippy warning baseline, application tests, the normal database/migration path, checked-in SQLx metadata, the standalone Rusty CLI regression suite, `./rusty audit`, dependency/supply-chain policy, production image/runtime smoke tests, and repository integrity.

The GitHub Actions production workflow is intentionally thin and delegates to this same project-owned gate.

### Stop the project

```bash
docker compose down
```

To also remove project data volumes:

```bash
docker compose down -v
```

Use `-v` deliberately: it removes named volumes, including PostgreSQL development data and Cargo/Node caches.

## Qualification evidence — historical POC baseline

### POC 0 — Green

The initial frontend/runtime baseline is working.

Verified:

- Rust 1.98.1 development toolchain
- Topcoat 0.9.0 at the original POC 0 checkpoint; the repository baseline was subsequently upgraded to Topcoat 0.10.0
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
- canonical application `./test` entrypoint
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

The current audit baseline covers validated domain input, UUID parsing, credential verification, persistence sink flows, dynamic SQL, shell execution, and related public-input boundaries. The production-readiness release run is green with no Rusty audit errors or warnings; the remaining finding is informational boundary evidence.

### View/DX Architecture v1 — Green baseline

View/DX Architecture v1 is adopted and exercised through real application presentation rather than remaining a feasibility-only experiment.

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
- the public home page renders through the runtime-view architecture
- HTML-first Contacts is the canonical and sole Contacts presentation surface under the established `/contacts/*` URLs
- successful ordinary form mutations use POST/Redirect/GET semantics while htmx requests receive targeted fragments
- unauthenticated HTTP behavior remains `/` = 200, `/login` = 200, `/contacts` = 401
- the native Topcoat login remains available at its established public URL
- native Topcoat procedures, components, shards, signals, and other reactive capabilities remain available as opt-in tools where their typed reactive model provides meaningful value
- Rusty provides `make:view`, `make:controller`, `make:view-model`, and atomic `make:page` scaffolding for the adopted HTML-first conventions

Renderer behavior is selected explicitly through `APP_ENV` rather than Rust compilation profile:

- `development` uses MiniJinja autoreload and rich server-side diagnostics
- `testing` uses a deterministic cached renderer with startup validation
- `production` uses a cached renderer, validates registered views before the HTTP server starts, disables MiniJinja rich debug mode, and restricts renderer logging to production-safe structured diagnostics
- renderer failures return Topcoat's generic HTTP 500 response rather than exposing template source, filesystem paths, or internal model details
- registered application views are validated through the project-owned view registry

The application pins and self-hosts htmx 2.0.10 for HTML-first server interactions where partial DOM replacement is appropriate. `HX-Request` is treated only as request/presentation metadata and never as an authentication or authorization boundary.

The Contacts migration proved that full-page HTML, htmx fragments, ordinary form fallbacks, typed application use cases, authentication, authorization, validation, and persistence can coexist behind stable public routes. Temporary `/view-dx/contacts*` migration routes and the `/native/contacts` control surface were retired after CRUD parity and progressive-enhancement behavior were established. `view-contacts-prg-v1` remains the rollback point immediately before native Contacts retirement.

## Current stack

### Application

- Rust 1.98.1
- Rust edition 2024
- Topcoat 0.10.0
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
- hardened production application, migration, and create-user images
- canonical `./release-gate` production-readiness matrix
- thin GitHub Actions production gate that delegates to the project-owned release gate

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

The decision is based on the completed renderer investigation and subsequent real application slices: external HTML renders through Topcoat, ordinary HTML edits can be observed without a Rust rebuild, typed Rust view models remain on the server side, HTML values are escaped by default, and renderer failures remain ordinary application errors rather than Rust compiler failures.

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
- production uses a fixed application-owned view root, disables rich template debug formatting, validates registered views at startup, and packages those views into the hardened read-only production runtime verified by `production-smoke`

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
- View/DX scaffolding for views, presentation controllers, view models, and complete public GET pages
- atomic `make:page` preflight/refusal behavior and dry-run support

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

### Potential Rusty extensions

Future Rusty work should only be added where it materially improves project consistency or development velocity.

The View/DX scaffolding baseline is already implemented through `make:view`, `make:controller`, `make:view-model`, and `make:page`.

Remaining candidate areas are:

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

## File-upload security policy

Uploads are unsupported by default.

The current application exposes no multipart route, file input, upload
storage abstraction, or document/attachment persistence model. Existing
browser mutations use URL-encoded forms and authentication procedures use
JSON. The application-wide request-body ceiling therefore remains 64 KiB and
must not be widened merely to accommodate a future upload feature.

A future upload or import capability must be introduced as an explicit,
security-reviewed route rather than implicitly enabling multipart handling
application-wide. At minimum, that route must:

- require the same explicit authentication and authorization appropriate to
  the uploaded resource;
- remain subject to the application's browser same-origin policy when it is a
  cookie-authenticated browser mutation;
- declare a route-scoped `BodyLimit::max(...)` sized for the specific use case
  rather than increasing or disabling the global request-body limit;
- prefer Topcoat's streaming multipart field interface for large file content
  rather than buffering an entire upload into memory without a separately
  justified bound;
- treat multipart field names, filenames, `Content-Type` values, and other
  client-supplied metadata as untrusted input;
- never use a client-supplied filename directly as a filesystem path or object
  storage key; storage identifiers must be generated by the application;
- validate allowed file types and content according to the feature's actual
  requirements rather than trusting filename extensions or declared MIME
  types alone;
- enforce explicit file/count/field limits in addition to the aggregate
  request limit when the feature permits multiple files or fields;
- store uploaded content outside executable application code and public static
  asset roots, with private access by default unless public exposure is an
  explicit product requirement;
- define cleanup behavior for rejected, interrupted, or partially persisted
  uploads so temporary or orphaned content is not silently retained;
- add malware scanning or quarantine before files become available whenever
  the accepted content types, threat model, or product requirements warrant
  it;
- avoid recording file contents, request bodies, credentials, session
  material, or client-supplied filenames in security logs by default; and
- include regression tests for authorization, origin enforcement, oversized
  bodies, malformed multipart input, filename/path traversal attempts,
  content-type mismatches, and storage cleanup as applicable.

Topcoat 0.9's `Multipart` extractor was verified during the earlier
upload-policy investigation as a streaming transport primitive that applied the
request's effective `BodyLimit` across the multipart stream. The repository now
targets Topcoat 0.10.0, so a future upload implementation must revalidate those
transport semantics against the pinned current Topcoat API before relying on
them. Framework multipart protections do not replace the application-level
filename, content-validation, authorization, storage, and lifecycle rules above.

## Rate-limiting policy

No blanket application-wide request limiter is enabled.

Rate limiting is applied where the protected resource, abuse model, and
identity key are known well enough to define a meaningful quota. A generic
global requests-per-minute limit would conflate unrelated operations, create
unnecessary contention for normal authenticated use, and can penalize many
users behind the same NAT or proxy.

The current authentication throttle remains a separate credential-defense
boundary:

- each normalized account identifier receives 5 login attempts per minute;
- each resolved client IP receives 30 login attempts per minute;
- the client-IP budget is consumed before arbitrary account keys can be
  created;
- missing client IP uses the existing shared fail-safe budget;
- oversized account identifiers share the bounded invalid-identifier key; and
- throttling occurs before database credential lookup and Argon2 verification.

That login throttle must not be replaced by a generic router-wide limiter.
Its account-plus-client model protects an unusually expensive anonymous
operation and deliberately avoids revealing whether an account exists.

Ordinary authenticated contact reads and CRUD mutations do not currently have
an additional application quota. They are already subject to authentication,
authorization, origin enforcement, request-size limits, and the normal
database/application boundaries. A future route-specific limiter should be
added only when a concrete resource-cost, abuse, automation, or business-rule
requirement justifies one.

When an application-facing route quota is required:

- prefer a stable authenticated user identifier as the primary key for
  authenticated operations instead of relying solely on IP address;
- use client IP only when it is relevant to the abuse model and the
  deployment's trusted-proxy configuration is explicit and verified;
- keep different resource classes in separate budgets rather than sharing one
  global counter;
- return Topcoat's typed HTTP 429 response with an appropriate `Retry-After`
  value rather than silently delaying requests;
- add a typed `RequestRateLimited` security event at the same time the first
  application-facing 429 quota is introduced;
- ensure rejected requests do not execute the expensive or state-changing
  operation they are intended to protect;
- bound attacker-controlled limiter keys and periodically discard stale keyed
  state where an in-process keyed limiter is used;
- test the allowed boundary, first rejected request, recovery after the
  window, key isolation, missing-key behavior, and response headers; and
- document whether the limiter is process-local, shared, or enforced at the
  edge.

Tower's generic `RateLimit` middleware is not the application quota primitive
for this baseline. Its service-readiness behavior applies backpressure by
waiting for capacity; it does not represent the explicit 429 rejection
semantics required for a caller-visible application quota. Tower concurrency
or rate controls may still be useful later as internal capacity/backpressure
tools where that behavior is intentional.

Client-IP policy is deployment-sensitive and deny-first. Topcoat trusts no reverse proxies by default, so the application uses the direct peer address and ignores forwarded client-address headers unless an explicit trusted-proxy boundary is configured.

Production deployments may configure `TRUSTED_PROXY_NETWORKS` as a comma-separated allowlist of exact proxy IP addresses and/or CIDR networks. The setting is loaded once at startup. An unset setting trusts no proxies; an explicitly empty, malformed, oversized, or catch-all configuration fails closed. Catch-all IPv4 and IPv6 networks such as `0.0.0.0/0` and `::/0` are not permitted.

When a request arrives from an explicitly trusted proxy network, Topcoat's trusted-proxy policy may use `X-Forwarded-For` to resolve the originating client address. The ingress proxy must therefore remove or overwrite any client-supplied `X-Forwarded-For` value and construct the forwarding chain from connection metadata it actually observes. A deployment must never add a proxy network to `TRUSTED_PROXY_NETWORKS` while allowing arbitrary client forwarding headers to pass through unchanged.

The resolved client address is security-sensitive application input. It is currently consumed by the authentication throttle and security-event telemetry. Trusted-proxy configuration must therefore reflect the real deployment topology rather than a broad private-network assumption, fixed hop count, or convenience default. Changes to proxy topology require corresponding review of the trusted-network allowlist.

`X-Forwarded-Host` and `X-Forwarded-Proto` are not application trust inputs in this baseline. The existing strict same-origin policy remains independent of trusted client-address resolution and must not be weakened merely because the application is deployed behind a proxy.

TLS terminates at the production ingress or reverse proxy rather than inside the application container. The ingress is responsible for accepting public HTTPS traffic, using an appropriately managed production certificate, and preventing ordinary public HTTP from reaching the application as an equivalent authenticated origin. The application container continues to serve its internal HTTP listener on the deployment network and must not be exposed directly to untrusted public traffic.

HTTP Strict Transport Security is likewise an ingress responsibility for this baseline. The production HTTPS ingress should emit the deployment's reviewed `Strict-Transport-Security` policy after HTTPS behavior is verified for the intended host scope. The application does not infer TLS state from forwarding headers and does not independently emit HSTS.

The ingress must preserve the externally authoritative request host information required by the application's browser same-origin boundary rather than substituting an unrelated internal upstream identity. Proxy routing, TLS termination, forwarded-client-address handling, and trusted-network configuration therefore form one deployment security boundary and must be validated together.

Anonymous/global traffic shaping and volumetric abuse protection remain primarily ingress responsibilities, where connection identity, deployment topology, and aggregate traffic are visible. Application-level quotas remain separate from this infrastructure boundary and should continue to use client IP only where the abuse model requires it and the deployment's trusted-proxy configuration is explicit and verified.

Process-local keyed limiters also do not constitute a globally authoritative
quota across multiple application replicas. Instance-local defensive limits
may use them where that scope is acceptable; globally consistent abuse,
billing, or business quotas require a shared enforcement point or a deliberate
edge/shared-store design.

Anonymous/global traffic shaping and volumetric abuse protection belong
primarily at the production ingress/reverse-proxy boundary, where connection
identity, deployment topology, and aggregate traffic are visible. POC 6
finalized the application-side contract: the application does not approximate a
globally authoritative edge quota, and deployment-specific traffic-shaping
policy remains an ingress/operations responsibility.

## Production error-handling policy

Expected user and business outcomes remain distinct from internal failures.

Validation failures, duplicate-email conditions, not-found outcomes,
authentication failures, authorization denials, request-size failures, and
other intentional HTTP/control-flow conditions are represented through their
typed application or Topcoat outcomes. They must not be converted into
internal-server errors merely because they are unsuccessful operations.

Unexpected application, persistence, authorization-storage, and rendering
failures cross the HTTP boundary as Topcoat's typed `InternalServerError`.
The browser receives only Topcoat's generic HTTP 500 response body:

`internal server error`

Database errors, SQL text, filesystem paths, template source, Rust debug
representations, error source chains, credentials, session material, request
bodies, query strings, and other backend diagnostics must not be copied into
that client response.

Unexpected Contacts persistence failures are not ordinary form or row states.
They propagate through the application/controller boundary and are converted
to the typed internal-server error only at the HTTP adapter. A database outage
or corrupted persistence operation must therefore not appear to the browser as
an ordinary successful HTTP response containing a validation-style message.

The application security-event layer classifies typed internal-server errors
as `RequestInternalError`. The event remains intentionally sanitized: it
records only the stable event kind, HTTP method, matched route pattern,
resolved client IP when available, and optional user ID. The underlying
backend error, raw URI/query, request body, credentials, cookies, session
material, and arbitrary headers are not event fields.

Normal `Result::Err` internal failures retain Topcoat's request context while
the framework converts them into the generic 500 response. The application's
security response headers therefore remain attached to those responses, and
the sanitized request-internal-error event can be emitted before Topcoat
performs the final client-facing conversion.

Panic handling remains a framework/runtime boundary. Topcoat 0.10 catches request-processing panics and converts them to generic HTTP 500 responses outside the normal completed application-layer path. The application therefore does not attempt to duplicate Topcoat's panic catcher or infer a normal request outcome after a panic.

Production installs a deliberately sanitized panic hook. A panic emits only:

`application_event kind=panic`

The production panic event does not include the panic payload, source location, backtrace, raw request data, backend error details, credentials, cookies, session material, or arbitrary headers. Production smoke testing proves that startup panics still fail closed while detailed panic text remains absent from logs.

Normal completed requests are observed by the application-owned request telemetry layer. Each `request_event` contains only the HTTP method, Topcoat matched route pattern, resolved client IP when available, response status when the application layer receives a completed response, an `ok` or `error` outcome, and elapsed milliseconds. Raw URI values, query strings, path-parameter values, request and response bodies, arbitrary headers, credentials, cookies, session material, and backend error values are excluded.

Typed application errors remain Topcoat errors until the framework converts them into their final HTTP responses. For those error paths, operational request telemetry records `outcome=error` with no fabricated status value, while the existing security-event layer independently records typed security-relevant failures.

The production health contract exposes two distinct unauthenticated endpoints:

- `GET /health/live` verifies only that the application process and router are responsive. It does not depend on PostgreSQL.
- `GET /health/ready` verifies the application's required PostgreSQL dependency through the application-layer readiness seam and its existing `SELECT 1` database healthcheck. Database failure returns HTTP 503 with a short `Retry-After` response rather than exposing database error details.

Liveness and readiness are intentionally separate. A temporary database outage must remove the instance from ready traffic without falsely declaring the application process dead.

Graceful shutdown is owned by Topcoat's server lifecycle. The application wraps the router in `RouterService` with an explicit 30-second shutdown timeout rather than relying silently on a framework default. The production image declares `STOPSIGNAL SIGTERM`; on SIGTERM, Topcoat stops accepting new connections and allows active requests to complete within that drain window. Normal shutdown emits `application_event kind=shutdown_complete`, and production smoke testing verifies a clean exit code of zero.

The application image intentionally does not define a Docker `HEALTHCHECK`. The minimal runtime image does not carry an HTTP client solely for probing, and a single image-level check would blur the distinction between liveness and dependency-backed readiness. The deployment platform should probe `/health/live` and `/health/ready` according to their separate semantics.

Server-side diagnostics remain deliberately bounded. Production code must not dump arbitrary backend errors with `Debug` formatting merely because the client response is sanitized. Operational telemetry uses explicit fields and excludes secrets and raw request payloads by default.


## Application audit logging policy

Durable application audit history is distinct from request/security telemetry.

The existing `SecurityEvent` stream records sanitized security decisions and
request-level failure context. Durable application audit rows instead answer a
narrow accountability question: which authenticated application user
successfully changed which protected business resource, and when.

The current durable audit scope is intentionally limited to successful
authenticated Contacts mutations:

- contact creation;
- contact update; and
- contact deletion.

Authentication/session lifecycle events remain security telemetry rather than
business audit rows. The trusted `create-user` operator workflow also remains
outside this audit scope because it does not currently execute as an
authenticated application user with an application actor identity.

Mutation authorization and audit attribution share the same typed capability.
`AuthorizationActor` retains the authenticated user's UUID, and each successful
Contacts mutation grant carries that actor UUID into the application use case.
Audit attribution is therefore not accepted as an independent web/form
parameter and cannot drift from the identity whose authorization granted the
mutation.

Each durable `public.audit_events` row contains only:

- a generated audit-event UUID;
- the authenticated actor user UUID;
- the typed mutation action (`create`, `update`, or `delete`);
- the fixed resource type (`contact`);
- the affected contact UUID; and
- the database-generated event timestamp.

The durable audit table deliberately excludes contact names and email
addresses, before/after snapshots, request bodies, raw URIs or query strings,
client IP addresses, cookies, credentials, session identifiers, token
material, arbitrary headers, and open-ended metadata.

Successful Contacts mutation persistence and its corresponding audit append
execute inside the same SQL transaction. The application commits only after
both operations succeed. If the audit append fails, the transaction is not
committed and the contact create, update, or delete is rolled back. Validation
failures, duplicate-email outcomes, not-found outcomes, and other unsuccessful
typed application outcomes do not create durable audit rows.

`actor_user_id` intentionally has no foreign key to `public.users`, and
`resource_id` intentionally has no foreign key to `public.contacts`. Durable
audit identifiers must survive future account retention/removal policy and
contact deletion rather than being deleted or made impossible by the source
row's lifecycle.

Schema migrations remain independent of deployment-specific PostgreSQL role
names so SQLx isolated test databases and migration-only environments can apply
the same schema. After migrations complete in the real deployment path,
`db::migrate::run` resets privileges on `public.audit_events` for
`topcoat_app` and restores `INSERT` only.

The runtime application therefore cannot select, update, delete, truncate,
reference, or attach triggers to durable audit history. Reading, exporting, or
retaining audit history requires a separate explicitly privileged operational
path rather than expanding the application's runtime database privileges.

The test contract proves successful actor/resource attribution, database action
and resource constraints, absence of foreign-key lifecycle coupling, exclusion
of unsuccessful application outcomes, and transaction rollback when audit
persistence fails for contact creation, update, or deletion.

Long-term audit retention, export, centralized collection, and operational access remain separate deployment and operations concerns. The completed production-readiness baseline provides sanitized application request and lifecycle telemetry plus release-gate verification, but it does not claim centralized log shipping, telemetry retention, an external metrics/alerting backend, or durable audit export.

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

The canonical complete application-suite command is:

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
cargo fmt --all -- --check
```

Linting:

```bash
cargo clippy --workspace --all-targets --all-features --locked
```

Run the complete safe application test suite:

```bash
./test
```

Run the full production-readiness matrix:

```bash
./release-gate
```

Inspect all containers, including the warm test database:

```bash
docker compose ps -a
```

Rusty CLI checks:

```bash
cargo fmt --manifest-path tools/rusty-cli/Cargo.toml --check
cargo check --manifest-path tools/rusty-cli/Cargo.toml --locked
cargo test --manifest-path tools/rusty-cli/Cargo.toml --locked
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

The development-loop behavior below was inspected directly on Topcoat 0.9 during the earlier View/DX investigation. The repository now targets Topcoat 0.10.0, so these observations are retained as historical compatibility evidence rather than represented as a fresh 0.10 verification.

Verified during that investigation:

- the previous successful application can keep serving while a rebuild is pending
- a failed rebuild does not require the previous working application to be torn down
- the dev broadcast process survives application restarts
- the browser protocol distinguishes rebuilding, build failure, application exit, reload, and already-up-to-date states
- successful reloads can refresh the current document through Topcoat's browser-side morph/update behavior rather than relying only on a blind full-page reload
- Topcoat watches local workspace sources broadly enough that runtime view directories may also trigger its watcher, even when Cargo ultimately has no Rust binary change to rebuild

`topcoat::dev::script()` remains part of the development integration because it provides Topcoat's browser-side development reload/status behavior.

During the 0.9 investigation, the dynamically selected development endpoint was not exposed correctly to the host browser under the Docker configuration and could produce a refused `dev.js` request. That did not affect application functionality in the tested configuration. Any future work on this development endpoint should first revalidate the behavior against the pinned Topcoat 0.10 baseline rather than assuming the 0.9 observation remains unchanged.

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

## Qualification history — POC 0 through POC 6

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
- `make:view`
- `make:controller`
- `make:view-model`
- atomic `make:page`
- View/DX scaffold preflight/refusal behavior and dry-run support

Potential future work:

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

**Status: Green v1 baseline**

View/DX Architecture v1 is adopted and exercised through real application presentation.

Implemented and verified:

- MiniJinja 2.24.0 provides runtime external HTML views through a narrow application-owned renderer boundary
- ordinary HTML view edits can be observed in development without a Rust rebuild
- runtime view failures remain ordinary application/runtime failures rather than compiler failures
- HTML-first presentation is the default for ordinary pages
- JavaScript, SCSS, CSS, images, and generated browser assets remain separate from Rust application code
- htmx 2.0.10 is pinned and self-hosted for server-owned partial interactions where that model is appropriate
- native Topcoat reactivity remains an opt-in capability rather than the required representation for ordinary presentation
- Rust controllers and application/query logic feed typed view models
- the canonical Contacts presentation uses the HTML-first architecture at its established public routes
- Rusty scaffolding conventions cover views, controllers, view models, and complete public GET pages without becoming a custom template or reactivity framework
- development, testing, and production use explicit view-loading, validation, diagnostic, and security behavior

Initial runtime-view and mixed-interaction feasibility was established during the Topcoat 0.9 phase. The repository's production and release baseline now targets Topcoat 0.10.0.

### POC 4 — Authorization

**Status: Green baseline for current POC scope**

The current authorization baseline is complete for the application's existing
resource model:

- explicit typed `viewer` and `operator` role policy;
- permission-specific authorization grants for Contacts read/create/update/delete;
- authorization enforced independently at externally callable Contacts routes/actions;
- presentation authorization omits mutation controls for read-only viewers;
- role-aware user provisioning supports the same persisted role model;
- negative application and HTTP authorization tests prove viewers cannot acquire
  or exercise Contacts mutation permissions; and
- authorization grants bind authenticated actor identity for mutation
  accountability without exposing session/token representation to application
  policy code.

Per-record ownership rules are not part of the current Contacts domain model and
are therefore not fabricated as an artificial authorization layer. If a future
resource introduces owner-scoped access, ownership must become an explicit
policy dimension with corresponding negative tests rather than being inferred
from the current role-only Contacts baseline.

### POC 5 — Web security hardening

**Status: Green baseline for current POC scope**

The application-wide web-security baseline is complete for the current POC:

- browser state-changing requests use an explicit strict same-origin policy with
  no trusted cross-origin peers or exempt mutation routes;
- CSP and response-security headers are applied through the shared application
  router stack;
- buffered request bodies have an explicit 64 KiB application-wide ceiling, with
  any future upload/import feature required to opt into a separately reviewed
  route-scoped limit;
- sanitized typed security events cover authentication/session decisions and
  relevant application/router failures without logging request payloads,
  credentials, session/token material, raw queries, or backend error details;
- uploads remain unsupported by default and are governed by the explicit
  file-upload security policy before any future multipart route is introduced;
- authentication retains its specialized account/client throttle, while broader
  route quotas follow the documented policy rather than applying an arbitrary
  blanket application limiter;
- unexpected application failures cross the HTTP boundary as generic typed 500
  responses while expected validation, duplicate, not-found, authentication,
  authorization, and request-limit outcomes retain their intended control-flow
  semantics; and
- successful authenticated Contacts create/update/delete operations append a
  minimal durable audit event in the same database transaction as the business
  mutation, with the runtime role restricted to INSERT-only audit access.

POC 5 intentionally deferred runtime and deployment concerns to POC 6 rather than emulating them inside application controllers. POC 6 now completes the production-readiness baseline for the current POC scope: reproducible dependency gates, production packaging, hardened container execution, reverse-proxy/TLS trust, operational request telemetry, sanitized panic handling, explicit health/readiness semantics, graceful process shutdown, and a canonical production release matrix.

Centralized log shipping and retention, external metrics/alerting infrastructure, registry selection, hosting-platform configuration, rollout strategy, and environment-specific ingress implementation remain deployment choices rather than application-framework responsibilities. The POC defines and verifies the contract those systems must preserve without pretending that one vendor-specific deployment is part of the framework.

### POC 6 — Production readiness

**Status: Green baseline for current POC scope**

Completed:

- dependency auditing and reproducibility release gates;
- production artifact/package contract;
- Topcoat 0.10 production baseline;
- production runtime/container hardening;
- trusted reverse-proxy configuration and forwarded-client-address policy;
- production TLS termination and HSTS ownership contract;
- sanitized whole-request operational telemetry;
- sanitized production panic telemetry;
- separate liveness and PostgreSQL-backed readiness endpoints;
- explicit SIGTERM handling with a 30-second Topcoat graceful-drain contract;
- production runtime-view and asset startup validation with fail-closed package checks;
- canonical `./release-gate` orchestration of formatting, locked all-target compilation, the exact tracked Clippy warning baseline, application tests, database/migration validation, SQLx metadata verification, Rusty CLI regression tests, Rusty architecture/security audit, dependency/supply-chain auditing, production runtime smoke testing, and repository-integrity checks; and
- `.github/workflows/production-gate.yml` as a thin GitHub Actions caller of the canonical release gate.

The GitHub Actions workflow runs for pull requests, pushes to `main`, and manual dispatch. It uses read-only repository contents permission, disables persisted checkout credentials, creates only ephemeral local PostgreSQL secret files required by the Docker test environment, consumes no repository deployment secrets, and delegates the actual validation matrix to `./release-gate` rather than duplicating release logic in YAML.

The complete qualification-era POC 6F release matrix has been exercised successfully in the project environment. The GitHub-hosted execution of the workflow remains an integration event that occurs when the workflow is pushed and triggered; the repository does not claim a hosted-run result that has not yet occurred.

Deployment remains deliberately vendor-neutral. `Dockerfile.prod` defines the production application, migration, and operator artifacts; `production-smoke` proves their packaging and runtime contract; and `./release-gate` is the deployability gate. A future deployment pipeline may publish and deploy those validated artifacts to a selected registry/platform, but it must not weaken the established runtime, proxy/TLS, health, secret, migration, or shutdown contracts.

## Qualification outcome

The original POC is complete.

Its question was not whether Rust can render a website. It was whether a Topcoat-based Rust application could meet a serious production bar while retaining reasonable framework ergonomics and ordinary frontend flexibility.

The result is:

> **Adopt With Constraints for Rust-heavy application contexts.**

The qualification work demonstrated:

- maintainable application/domain/persistence separation;
- secure authentication and session architecture;
- typed authorization;
- strict browser-origin and response-security policy;
- bounded request handling and explicit upload policy;
- application audit logging;
- runtime external HTML views with ordinary frontend editing;
- progressive enhancement through htmx;
- opt-in native Topcoat reactivity;
- isolated PostgreSQL integration testing;
- SQLx compile-time contracts;
- project-owned dependency and security auditing;
- hardened production images;
- trusted-proxy/TLS contracts;
- sanitized operational telemetry;
- liveness/readiness semantics;
- graceful shutdown;
- reproducible production packaging;
- a canonical release gate and thin CI caller.

The qualification also established the principal constraint: Topcoat is not yet as mature or batteries-included across common application concerns as a framework such as Laravel. Atlasyr therefore accepts an ongoing platform-maintenance responsibility and must continuously reassess whether project-owned features should remain owned, become provider choices, or migrate to new Topcoat capabilities.

### Qualification scorecard

The final framework evaluation used Laravel + Livewire as the mature reference bar rather than as a product target.

| Decision criterion | Atlasyr qualification baseline | Laravel + Livewire reference |
| --- | ---: | ---: |
| Maintainable application organization | 4.5 / 5 | 4.5 / 5 |
| Secure defaults or explicit replacements | 4.0 / 5 | 5.0 / 5 |
| Reasonable development velocity | 3.0 / 5 | 5.0 / 5 |
| Predictable framework behavior | 4.0 / 5 | 5.0 / 5 |
| Strong testing ergonomics | 4.5 / 5 | 5.0 / 5 |
| Manageable dependency risk | 4.0 / 5 | 4.5 / 5 |
| Clean deployment and operations | 4.5 / 5 | 5.0 / 5 |
| Ordinary HTML/CSS/JavaScript without unnecessary Rust recompilation | 4.5 / 5 | 5.0 / 5 |
| Progressive standard-web to native reactive path | 5.0 / 5 | 5.0 / 5 |
| Clear separation of concerns | 5.0 / 5 | 4.5 / 5 |

The score is evidence for where Atlasyr should invest next, not a marketing benchmark.

### Adoption constraints

Atlasyr proceeds under these constraints:

- Topcoat remains explicitly credited as the upstream framework foundation.
- Topcoat upgrades are handled through dedicated compatibility review, not casual dependency bumps.
- ordinary presentation remains HTML-first unless native Topcoat machinery provides a meaningful advantage;
- native Topcoat remains available rather than being hidden behind an Atlasyr imitation;
- framework-owned security behavior remains explicit, tested, and fail-closed;
- mature third-party infrastructure is integrated rather than recreated;
- Rusty automates established conventions rather than inventing a parallel framework;
- core simplification is preferred over generating unnecessary boilerplate;
- new abstractions require pressure from real application slices;
- production and dependency release gates remain part of the framework contract.

### Immediate closeout work

The product roadmap begins with several qualification-closeout tasks:

1. record and freeze this final qualification decision;
2. review the inherited `MAX_TRUSTED_PROXY_NETWORKS = 64` limit and either justify it with evidence or remove the arbitrary cap in a focused corrective slice;
3. revalidate the historical Topcoat 0.9 Docker `dev.js` development-endpoint observation against the pinned Topcoat 0.10 baseline;
4. seed the Topcoat/Atlasyr capability ledger;
5. add durable upstream attribution / notice material appropriate to Topcoat's MIT-licensed foundation;
6. plan the repository/package/product rename without rewriting frozen qualification history;
7. establish the permanent Topcoat compatibility workflow before major new product features.

After that boundary is frozen, future work is product/framework development rather than additional numbered POCs.

## Historical qualification record

The detailed POC 0 through POC 6 record above is intentionally retained as engineering evidence. Those sections document why current Atlasyr conventions exist and provide rollback/reference points for future framework changes.

The forward-looking roadmap is no longer maintained as a POC sequence. See [`ROADMAP.md`](ROADMAP.md).
