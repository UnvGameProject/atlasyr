# Atlasyr Roadmap

**Rust at the core. Web without the detour.**

This roadmap begins after the completed Topcoat qualification POC. It is a product roadmap, not a continuation of numbered proof-of-concept phases.

Atlasyr's objective is to become the web application framework Rust-heavy teams reach for when the important part of their system is already Rust.

The roadmap has no calendar deadline. Work advances when its technical exit criteria are satisfied.

## Roadmap rules

Every roadmap decision follows these rules:

1. **Topcoat is upstream, not a fixed dependency snapshot.** Atlasyr must expect Topcoat to add features, remove APIs, and change architecture while it remains pre-1.0.
2. **Upstream before duplication.** Before building a capability, compare it with released Topcoat, Topcoat `main`, and the upstream roadmap.
3. **Simplify before scaffolding.** Reduce unnecessary framework ceremony before teaching Rusty to generate it.
4. **Real application pressure before abstraction.** New core abstractions need evidence from real feature work.
5. **Provider choice must represent a real use-case difference.** Atlasyr should not keep duplicate implementations merely because both exist.
6. **Delete code when upstream makes it unnecessary.** Removal is a successful outcome.
7. **Security remains explicit.** Rust memory safety does not replace application-security controls.
8. **Mature infrastructure stays external.** Atlasyr integrates with databases, brokers, object stores, and observability systems rather than reimplementing them.
9. **Advanced capabilities stay optional.** Distributed systems, brokers, WASM, WebGPU, and specialized data stores must not bloat the minimum application.
10. **Production behavior is part of the framework contract.** Release gates, shutdown behavior, health semantics, dependency integrity, and deployment boundaries remain continuously tested.

## Capability ownership model

Every cross-cutting feature should appear in an Atlasyr/Topcoat capability ledger.

| State | Definition | Preferred action |
| --- | --- | --- |
| **Topcoat-native** | Topcoat already provides the preferred capability. | Use it directly unless Atlasyr needs a narrow policy boundary. |
| **Atlasyr adapter** | Topcoat provides the primitive; Atlasyr adds policy, compatibility, or integration. | Keep the adapter narrow and deletable. |
| **Atlasyr-owned** | The capability belongs to Atlasyr or does not exist upstream. | Maintain it with explicit tests and ownership. |
| **Selectable provider** | Different implementations solve meaningfully different problems. | Define a stable provider seam and document tradeoffs. |
| **Experimental** | The capability is being evaluated. | Keep it out of stable framework guarantees. |
| **Deprecated / upstream-replaced** | Topcoat now provides the preferred implementation. | Publish a migration path, then remove duplication. |

Provider choice is not a goal by itself. If Topcoat's future implementation fully supersedes an Atlasyr implementation, Atlasyr should normally migrate and delete its copy. If both implementations serve legitimately different workloads, Atlasyr may retain them behind a provider boundary.

## Permanent upstream watch

Topcoat currently describes itself as early-stage and experimental, and its roadmap overlaps several areas Atlasyr may otherwise build: validation, authentication, background jobs, easier rate limiting, OpenAPI, localization, deployment documentation, WebTransport, image optimization, islands, static export, and additional reactivity/UI work.

Those upstream items must remain visible in Atlasyr planning.

No roadmap task in an overlapping area begins implementation until its capability-ledger entry records one of:

- **adopt upstream**;
- **adapt upstream**;
- **coexist as providers**;
- **defer for upstream**;
- **Atlasyr-specific implementation justified**.

---

# Milestone 0 — Qualification closeout and product transition

**Priority: immediate**

This milestone closes the POC cleanly and establishes Atlasyr as the product identity without rewriting historical evidence.

## 0.1 Record the final decision

- Add the final **Adopt With Constraints** evaluation to project documentation.
- Preserve the Laravel + Livewire comparison as a maturity bar, not a replacement goal.
- Preserve the ten qualification criteria and scores as baseline evidence.
- Freeze a final qualification/evaluation checkpoint after documentation and corrective work are complete.
- Do not create POC 7.

## 0.2 Correct inherited qualification debt

### Trusted-proxy network cap

Review:

```rust
const MAX_TRUSTED_PROXY_NETWORKS: usize = 64;
```

Tasks:

- identify whether Topcoat, parsing complexity, memory use, or a measured deployment constraint justifies the cap;
- if evidence supports a bound, document the threat/resource model and tests;
- if no evidence supports `64`, remove or redesign the arbitrary limit;
- preserve catch-all rejection and malformed-input fail-closed behavior;
- rerun only the focused proxy/security validation required by the corrective slice.

Exit criterion: no arbitrary security limit remains merely because it sounds defensive.

### Historical `dev.js` Docker behavior

- reproduce the historical Topcoat 0.9 development-endpoint issue against Topcoat 0.10;
- inspect current Topcoat CLI/runtime development behavior before changing Atlasyr;
- classify the issue as resolved upstream, configuration error, current Topcoat limitation, or Atlasyr integration gap;
- only implement a fix if the 0.10 baseline still demonstrates the problem;
- add a regression check if Atlasyr owns the fix.

Exit criterion: README no longer depends on an unverified 0.9 development-loop assumption.

## 0.3 Product identity and attribution

- use **Atlasyr** as the working product identity;
- perform trademark, domain, crates.io, GitHub organization/repository, package-manager, and major software-product clearance before public branding;
- add durable "Built on Topcoat" attribution;
- add a `NOTICE` or equivalent attribution document appropriate to Topcoat's MIT license and Atlasyr's distribution model;
- state clearly that Atlasyr is independent and not an official Tokio/Topcoat project;
- decide which historical names stay visible only in Git history and qualification notes.

## 0.4 Repository transition plan

Plan, then execute separately:

- repository rename from the qualification-era `topcoat_poc` name;
- crate/package naming;
- Docker image names;
- Compose project/service naming where appropriate;
- CI workflow names;
- documentation paths;
- Rusty CLI naming decision;
- GitHub organization/repository naming;
- generated-template defaults.

Do not rewrite frozen Git history or existing tags.

## 0.5 Seed the capability ledger

Create the first explicit Atlasyr/Topcoat ledger covering at minimum:

- routing;
- request context;
- cookies;
- sessions;
- authentication;
- authorization;
- validation;
- runtime rendering;
- external HTML views;
- htmx;
- native Topcoat reactivity;
- assets;
- mail;
- rate limiting;
- background jobs;
- uploads/multipart;
- OpenAPI;
- localization;
- realtime transports;
- deployment;
- health/shutdown;
- observability.

For every capability record:

- current Atlasyr implementation;
- current Topcoat implementation;
- current Topcoat roadmap status if applicable;
- security implications;
- ownership state;
- migration/deprecation trigger.

### Milestone 0 exit criteria

- final qualification decision recorded;
- proxy cap resolved or explicitly justified;
- Topcoat 0.10 development-loop behavior revalidated;
- capability ledger exists;
- Topcoat attribution/notice exists;
- product naming transition is documented;
- final qualification tree is clean and tagged.

---

# Milestone 1 — Topcoat upstream-sync and compatibility spine

**Priority: immediately after closeout**

This milestone makes Topcoat movement cheap enough that Atlasyr can safely build on a fast-moving upstream.

## 1.1 Compatibility policy

Define:

- Atlasyr versioning independent from Topcoat versioning;
- while Topcoat is pre-1.0, stable Atlasyr releases pin an exact qualified Topcoat version unless evidence supports a wider range;
- every Atlasyr release records its supported Topcoat version(s);
- a Topcoat upgrade is never a blind dependency bump;
- Atlasyr can ship a compatibility-only patch/minor release when upstream changes require it.

Create a compatibility matrix containing:

- Atlasyr version;
- Topcoat version;
- Rust MSRV;
- pinned development toolchain;
- SQLx/PostgreSQL baseline where relevant;
- migration notes;
- known limitations.

## 1.2 Release-intake workflow

For each Topcoat release:

1. retrieve release notes/changelog;
2. compare `Cargo.toml`, relevant public APIs, examples, and docs;
3. identify breaking changes;
4. inspect CLI/runtime version coupling;
5. review Rust/MSRV changes;
6. inspect session/cookie/request/security-sensitive changes;
7. inspect router/middleware changes;
8. inspect asset/runtime/browser changes;
9. inspect htmx/integration changes;
10. compare upstream additions with Atlasyr-owned features;
11. run compatibility tests;
12. classify overlapping features;
13. update docs/capability ledger;
14. migrate/delete Atlasyr code where appropriate;
15. run the canonical release gate;
16. record the supported-version decision.

## 1.3 Topcoat `main` canary

Add a non-release CI path that periodically or manually tests Atlasyr against Topcoat `main`.

Requirements:

- never publish a stable Atlasyr release from the canary;
- failure must not automatically break the pinned stable branch;
- report source/API breakage clearly;
- focus on Atlasyr-consumed Topcoat surfaces;
- permit expected failures with an explicit tracked reason, not a blanket ignore;
- make it easy to open a compatibility branch before the upstream release lands.

The canary should detect likely breakage early, not force Atlasyr to chase every upstream commit.

## 1.4 Compatibility contract tests

Build focused tests around the Topcoat surfaces Atlasyr relies on most:

- route registration and matched-route behavior;
- request context;
- cookies;
- sessions and lifecycle behavior;
- client-IP/trusted-proxy handling;
- error conversion and panic behavior;
- assets and production asset loading;
- runtime startup;
- external HTML/Topcoat response bridge;
- htmx request/response helpers;
- graceful shutdown;
- mail if adopted;
- WebSocket/SSE/WebTransport when adopted.

Prefer contract tests over broad wrappers.

## 1.5 Upstream contribution policy

Before Atlasyr permanently works around a generic Topcoat problem:

- decide whether the behavior is Atlasyr-specific;
- if generic, discuss/submit upstream where appropriate;
- keep any temporary compatibility patch isolated and deletable;
- link Atlasyr workaround removal to the upstream issue/release when possible.

### Milestone 1 exit criteria

- Topcoat upgrades follow a repeatable documented process;
- a compatibility matrix exists;
- the capability ledger is maintained;
- stable and canary upstream paths are distinct;
- contract tests cover Atlasyr's important upstream dependencies;
- upstream overlap can be discovered before major Atlasyr implementation begins.

---

# Milestone 2 — Development Velocity v1

**Primary product goal: materially improve the 3.0/5 qualification velocity score for the intended Rust-heavy user**

This work deliberately precedes broad new infrastructure integrations.

## 2.1 Choose a representative second feature

Do not use Contacts again.

Choose a feature that exercises a realistic vertical slice, for example a `projects` or `workspaces` resource with:

- migration/schema;
- persisted model;
- query/persistence layer;
- create/update/delete/list/read;
- validation;
- authentication;
- authorization;
- durable audit behavior;
- HTML-first page;
- htmx fragment behavior;
- tests;
- navigation;
- production/release compatibility.

The exact resource is less important than exercising the framework as a user would.

## 2.2 Measure the current workflow

Before improving anything, record:

- files created manually;
- registrations edited manually;
- repetitive types/impls;
- commands required;
- compiler/test cycles;
- Rust rebuilds caused by frontend-only changes;
- errors that require framework-source knowledge;
- places where Topcoat knowledge leaks unnecessarily into domain/application code;
- places where security controls are easy to forget;
- time/steps spent on non-domain wiring.

Do not establish arbitrary performance targets until the baseline exists.

## 2.3 Core simplification pass

For each repetitive step ask:

- can the API make this unnecessary?
- can a sensible default remove a choice?
- can a typed helper/error conversion remove boilerplate?
- does an existing Topcoat API already solve it?
- is the repetition an intentional security boundary that should remain visible?

Potential core areas:

- controller-to-application error mapping;
- authenticated actor extraction;
- authorization grant plumbing;
- common view response construction;
- typed form/validation outcome mapping;
- PRG helpers;
- htmx/full-page dual-response helpers;
- audit transaction composition;
- route registration conventions;
- configuration diagnostics.

Avoid macros unless repeated evidence shows they provide a better contract than ordinary Rust.

## 2.4 Rusty expansion

Only after the simplification pass, evaluate generators/commands such as:

- `make:feature`;
- `make:resource`;
- `make:authenticated-page`;
- `make:form`;
- `make:policy`;
- `make:use-case`;
- `make:query`;
- `make:integration`;
- `model:inspect`;
- `model:sync`;
- `doctor:compat`;
- test scaffolds for application, HTTP, authorization, and persistence boundaries.

Every generator must retain existing Rusty properties:

- full preflight before writes;
- deterministic output;
- refusal on collisions;
- path/name validation;
- no hidden security assumptions;
- `--dry-run`;
- rollback if a multi-file plan fails;
- regression tests.

Rusty should not become a source-code parser or a shadow framework when a better stable registration mechanism can exist in core.

## 2.5 Diagnostics and developer feedback

Improve:

- startup configuration errors;
- missing-view errors;
- invalid provider errors;
- migration/schema mismatch guidance;
- SQLx metadata diagnostics;
- release-gate failure summaries;
- Rusty scaffold collision messages;
- compatibility-version mismatch diagnostics.

## 2.6 Re-measure the feature

Repeat the same feature workflow after changes and compare:

- manual wiring steps;
- non-domain files touched;
- commands;
- feedback-loop latency;
- failure discoverability;
- code volume that exists only to satisfy framework ceremony.

### Milestone 2 exit criteria

- common feature development is materially faster than the qualification baseline;
- improvements come from core simplification and tooling, not hidden magic;
- security/application boundaries remain explicit;
- frontend-only edits still avoid unnecessary Rust rebuilds;
- velocity claims are backed by a documented before/after workflow.

---

# Milestone 3 — Framework core stabilization

This milestone turns the qualified application architecture into a deliberately consumable framework.

## 3.1 Distribution shape

Decide whether Atlasyr is primarily:

- a template;
- a Cargo workspace with reusable Atlasyr crates;
- a small core crate plus template;
- a provider-oriented collection of crates.

Avoid prematurely extracting code that has only one application consumer.

## 3.2 Stable application contracts

Identify and document stable seams for:

- configuration;
- application lifecycle;
- authentication identity;
- authorization policy;
- application errors;
- view rendering;
- request telemetry;
- health/readiness;
- persistence/application transaction patterns;
- provider registration.

Keep Topcoat-specific types at adapters when application/domain layers do not need them.

## 3.3 Provider model

Define a provider model only for capabilities that genuinely need alternate implementations.

Provider requirements:

- startup selection unless runtime selection is a real product requirement;
- exact supported implementations;
- fail closed on unknown provider;
- no request-controlled security-provider switching;
- provider-specific health/readiness where appropriate;
- capability tests shared across providers.

## 3.4 Public API discipline

- establish Atlasyr SemVer policy;
- establish deprecation policy;
- document stable vs experimental modules;
- minimize public surface;
- avoid re-exporting entire dependencies casually;
- document MSRV;
- document compatibility with Topcoat;
- add API examples and compile tests.

### Milestone 3 exit criteria

- Atlasyr has a deliberate distribution model;
- stable core seams are identified;
- provider boundaries are justified;
- application code is not accidentally coupled to internal framework details;
- compatibility/deprecation policy is published.

---

# Milestone 4 — Web application essentials

Build common web capabilities where they help Rust-heavy applications, but treat Topcoat roadmap overlap as a gating input.

## 4.1 Validation and forms

**Topcoat watch: validation is on the upstream roadmap.**

Before expanding Atlasyr validation:

- inspect the current Topcoat validation design;
- decide adopt/adapt/coexist/defer;
- preserve field-specific normalization;
- preserve server-authoritative validation;
- standardize typed form-error presentation;
- support ordinary full-page and htmx outcomes.

## 4.2 Authentication and account lifecycle

**Topcoat watch: authentication is on the upstream roadmap.**

Evaluate:

- current Atlasyr local authentication;
- future Topcoat authentication;
- OIDC/OAuth providers;
- public registration;
- password reset;
- email verification;
- MFA/passkeys/WebAuthn;
- session rotation.

Do not keep a custom local implementation merely for ownership if upstream becomes stronger and compatible.

## 4.3 Authorization

Extend only from real domains:

- resource policies;
- owner-scoped access;
- organization/tenant policy;
- permission composition;
- policy scaffolding;
- negative authorization tests.

## 4.4 Mail and notifications

Topcoat already has mail support.

Tasks:

- evaluate direct Topcoat use;
- build Atlasyr notification conventions only where application policy adds value;
- support test/in-memory delivery;
- consider async delivery only after the background-job decision.

## 4.5 Uploads and storage

Before implementation:

- revalidate Topcoat 0.10+ multipart streaming behavior;
- preserve route-scoped body limits;
- generated storage keys;
- content/type validation;
- private-by-default storage;
- cleanup of partial/rejected content;
- malware/quarantine hooks when threat model requires;
- object-storage providers;
- authorization and audit tests.

## 4.6 Rate limiting

**Topcoat watch: easier rate limiting is on the upstream roadmap.**

Maintain distinction between:

- credential-defense throttling;
- application resource quotas;
- internal backpressure/concurrency;
- edge/volumetric traffic control.

Evaluate future Topcoat rate-limiting helpers before expanding Atlasyr abstractions.

## 4.7 OpenAPI

**Topcoat watch: OpenAPI endpoints are on the upstream roadmap.**

Do not build a parallel OpenAPI system until upstream direction is clear.

## 4.8 Localization

**Topcoat watch: localization is on the upstream roadmap.**

Defer framework-level localization until an application requirement and upstream review exist.

## 4.9 Image optimization

**Topcoat watch: image optimization/resizing is on the upstream roadmap.**

Prefer upstream or mature image tooling; keep application-specific pipelines optional.

### Milestone 4 exit criteria

- common web essentials are easy for Rust teams;
- no major feature duplicates an upstream Topcoat feature without a recorded reason;
- security defaults remain explicit and tested.

---

# Milestone 5 — Data and storage integrations

PostgreSQL + SQLx remains the reference relational path, but Atlasyr should make other serious Rust data systems easy to integrate.

## 5.1 PostgreSQL/SQLx deepening

- retain compile-time query contracts;
- improve transaction composition;
- model/schema inspection;
- Rusty `model:inspect` / `model:sync` experiments;
- generate only from real database metadata;
- use actual primary-key metadata;
- map known PostgreSQL types explicitly;
- fail on unknown types rather than guessing;
- support read replicas only when a real consistency model is documented.

## 5.2 Redis

Potential capabilities:

- cache;
- distributed locks where justified;
- shared quota state;
- pub/sub or streams when appropriate;
- session/provider experiments only if they improve on existing session requirements.

## 5.3 MongoDB

Use the official/mature Rust driver.

Atlasyr may provide:

- connection/configuration conventions;
- health/readiness;
- typed repository/application patterns;
- tenant/database routing;
- topology-aware access;
- shard-key conventions;
- observability.

Atlasyr must not pretend to implement MongoDB sharding itself.

## 5.4 Object storage

Support S3-compatible storage patterns for:

- uploads;
- generated assets;
- reports;
- compute artifacts.

Keep public/private access policy explicit.

## 5.5 Multi-tenancy

Only after a real tenant model exists:

- tenant resolution;
- authorization coupling;
- per-tenant data routing;
- schema/database/shard strategies;
- audit attribution;
- cache key isolation;
- job/message propagation.

### Milestone 5 exit criteria

- PostgreSQL remains excellent;
- additional stores plug in without contaminating domain logic;
- topology/consistency decisions stay visible rather than hidden by a false universal repository abstraction.

---

# Milestone 6 — Async, messaging, workers, and realtime

This is a major Atlasyr differentiation area because Rust-heavy systems frequently already depend on Tokio and message-driven architecture.

## 6.1 Async task lifecycle

Define framework conventions for:

- cancellation;
- shutdown;
- timeouts/deadlines;
- backpressure;
- bounded concurrency;
- task ownership;
- error propagation;
- task telemetry.

Avoid detached "fire-and-forget" tasks as a default.

## 6.2 Broker integrations

Evaluate provider crates/adapters for:

- RabbitMQ;
- NATS / JetStream;
- Kafka;
- Redis Streams.

Common patterns:

- producer/consumer lifecycle;
- connection health;
- graceful drain;
- retry/backoff;
- dead-letter strategy;
- idempotency;
- message tracing;
- typed serialization;
- poison-message handling.

## 6.3 Transactional messaging

Provide documented patterns for:

- transactional outbox;
- inbox/deduplication;
- idempotency keys;
- exactly-once claims avoidance;
- replay;
- audit correlation.

## 6.4 Background jobs

**Topcoat watch: background jobs are on the upstream roadmap.**

Before Atlasyr builds a job framework:

- inspect released/upcoming Topcoat job APIs;
- determine whether Atlasyr needs only providers/adapters;
- if Atlasyr already has worker primitives, decide whether they remain useful for broker-backed or distributed jobs;
- do not retain two equivalent job systems indefinitely.

## 6.5 Realtime transports

Evaluate and standardize:

- WebSockets;
- server-sent events;
- Topcoat/Datastar where applicable;
- streaming responses;
- binary payloads.

**Topcoat watch: WebTransport is on the upstream roadmap.**

## 6.6 Reference application

Build a reference message-driven feature proving:

- web request -> durable state -> outbox -> broker -> worker -> state update -> realtime/browser observation;
- retries and idempotency;
- graceful shutdown;
- telemetry;
- failure recovery.

### Milestone 6 exit criteria

- Atlasyr can expose Rust async/message-driven systems naturally;
- worker and web lifecycles cooperate;
- broker choice remains optional;
- Topcoat background-job/realtime evolution is incorporated rather than ignored.

---

# Milestone 7 — Observability and reliability

## 7.1 Tracing and OpenTelemetry

- adopt `tracing` conventions consistently;
- evaluate OpenTelemetry exporters;
- trace propagation across HTTP, jobs, and brokers;
- sanitized structured fields;
- request/job/message correlation;
- no secret or raw payload logging by default.

## 7.2 Metrics

Define optional metrics for:

- HTTP latency/error rate;
- queue depth/consumer lag where available;
- database pool health;
- background-job outcomes;
- readiness;
- runtime/task saturation where useful.

Avoid high-cardinality labels by default.

## 7.3 Audit export

Keep durable business audit events distinct from operational telemetry.

Add an explicitly privileged export/retention path without expanding runtime application DB privileges.

## 7.4 Resilience patterns

Add only where demonstrated by real integrations:

- timeout/deadline budgets;
- retries with jitter;
- circuit breakers;
- bulkheads;
- bounded queues;
- load shedding;
- dependency-specific readiness.

Do not add decorative resilience abstractions without failure-mode evidence.

## 7.5 Fault testing

- dependency outage tests;
- broker interruption;
- DB failover scenarios;
- shutdown while work is active;
- slow dependency behavior;
- invalid/stale cache behavior;
- multi-service trace continuity.

### Milestone 7 exit criteria

- production incidents can be understood without leaking sensitive data;
- web, worker, and messaging lifecycles are observable;
- reliability controls correspond to demonstrated failure modes.

---

# Milestone 8 — Security and identity expansion

## 8.1 Identity providers

Potential providers:

- local credentials;
- OIDC;
- OAuth2;
- enterprise SSO/SAML only when justified;
- passkeys/WebAuthn.

Provider architecture must preserve a stable application identity model.

## 8.2 Secrets and key management

Evaluate:

- Docker/Kubernetes secrets;
- Vault;
- cloud KMS/secret managers;
- key rotation;
- signing/encryption key lifecycle.

No secret provider should be required for local development.

## 8.3 Security test kit

Expand reusable tests for:

- authorization denials;
- origin/CSRF policy;
- session fixation/rotation;
- credential enumeration;
- rate-limit behavior;
- upload path/content attacks;
- output escaping;
- trusted-proxy spoofing;
- redirect safety;
- cache/tenant isolation.

## 8.4 Fuzz/property testing

Use where input parsers or security-sensitive state machines justify it.

## 8.5 Supply-chain/release integrity

Extend current dependency gates toward:

- SBOM generation;
- provenance/attestation;
- signed release artifacts where distribution model warrants;
- vulnerability-response workflow;
- security disclosure policy;
- reproducible build evidence where practical.

### Milestone 8 exit criteria

- common identity/security extensions have explicit threat models;
- provider growth does not weaken the original fail-closed baseline;
- release integrity scales with public distribution.

---

# Milestone 9 — Global and distributed systems

This track targets systems where Atlasyr should differentiate from ordinary web-framework assumptions.

## 9.1 Multi-region reference architecture

Document:

- stateless HTTP placement;
- region-local vs global state;
- readiness/failover;
- ingress and edge behavior;
- DNS/routing assumptions;
- data residency;
- secret/key locality.

## 9.2 Data consistency

Make consistency models explicit:

- strong vs eventual reads;
- read-your-writes;
- replica lag;
- conflict resolution;
- idempotency;
- retry safety.

Do not hide distributed tradeoffs behind a generic repository API.

## 9.3 Tenant/shard routing

Provide optional patterns for:

- tenant -> region;
- tenant -> database;
- tenant -> shard;
- read/write route selection;
- routing metadata propagation;
- cache isolation;
- audit/telemetry attribution.

## 9.4 Distributed coordination

Only when required:

- leader election;
- distributed locks;
- leases;
- fencing tokens;
- consensus-backed systems.

Prefer mature external systems.

## 9.5 Global messaging

Document:

- event ownership;
- replication;
- cross-region retries;
- duplicate delivery;
- ordering;
- replay;
- regional failure behavior.

### Milestone 9 exit criteria

- Atlasyr can support global Rust systems without pretending distributed systems are transparent;
- tenant/region/shard decisions remain testable and observable.

---

# Milestone 10 — Advanced compute, WASM, WebGPU, and graphics

This is optional differentiation, not core-web-framework scope.

## 10.1 Rust-to-browser compute bridge

Investigate:

- WASM modules for compute-heavy client work;
- shared schema/type generation where it genuinely helps;
- binary transport;
- streaming datasets/results;
- worker threads where browser support permits;
- memory/asset lifecycle.

Do not move logic client-side merely because Rust can compile to WASM.

## 10.2 WebGPU

Build an optional integration/reference path for:

- high-performance visualization;
- geometry;
- simulation;
- data rendering;
- GPU compute where appropriate.

## 10.3 3D reference application

Use mature browser graphics formats/tooling such as glTF/WebGPU ecosystems.

A useful reference would prove:

- Rust server-side geometry/compute;
- efficient asset/result transport;
- browser visualization;
- progressive loading;
- authorization of expensive compute;
- job progress;
- caching/storage;
- performance telemetry.

Do not market "Blender fidelity in the browser" as a framework property. Visual fidelity depends on rendering algorithms, assets, shaders, GPU/browser capabilities, and resource budgets.

## 10.4 Compute-job lifecycle

Integrate with the async/worker track:

- submit;
- validate/authorize;
- schedule;
- cancel;
- stream progress;
- persist artifacts;
- expire/clean up;
- audit.

### Milestone 10 exit criteria

- advanced compute capabilities plug into Atlasyr without distorting the core framework;
- at least one reproducible reference workload demonstrates why a Rust-centric web platform is valuable.

---

# Milestone 11 — Provider and integration ecosystem

## 11.1 Crate/package strategy

Potential future split, only after core stabilization:

- Atlasyr core;
- Topcoat compatibility/integration crate if a real seam is required;
- broker providers;
- storage providers;
- observability providers;
- authentication providers;
- test-support crate;
- Rusty CLI.

Do not split crates merely for branding.

## 11.2 Capability tests

Each provider family should share contract tests where meaningful:

- startup/configuration;
- health/readiness;
- expected success/failure semantics;
- shutdown;
- security behavior;
- observability.

## 11.3 Third-party extension guidance

Document:

- stable extension points;
- provider naming;
- compatibility metadata;
- security expectations;
- test requirements;
- versioning expectations.

## 11.4 Upstream vs ecosystem decision

Generic web-framework features that fit Topcoat should be proposed upstream when appropriate.

Atlasyr-specific Rust-heavy integrations should normally remain in Atlasyr/provider crates.

### Milestone 11 exit criteria

- extensions can grow without turning Atlasyr core into a dependency bundle;
- third-party providers have a testable contract;
- Topcoat remains visible rather than buried under a permanent compatibility façade.

---

# Milestone 12 — Distribution, documentation, and maturity

## 12.1 Public naming clearance

Before public launch:

- trademark search;
- domain search;
- crates.io package availability;
- GitHub organization/repository availability;
- container/image namespace availability;
- major package/software collision review.

If **Atlasyr** cannot be cleared, preserve the product thesis and choose a new name before public release rather than accruing avoidable naming debt.

## 12.2 CLI identity

Decide whether:

- `Rusty` remains the developer-tool brand;
- Rusty becomes `atlasyr`;
- `atlasyr` becomes a top-level CLI with `rusty` compatibility aliases.

Base the decision on actual product usability and migration cost.

## 12.3 Starter profiles

Potential profiles:

- HTML-first application;
- native Topcoat reactive application;
- mixed application;
- API/service + operator UI;
- message-driven worker + web control plane.

Profiles should share the same security/production foundation.

## 12.4 Documentation system

Build:

- getting started;
- architecture guide;
- security guide;
- deployment guide;
- Topcoat compatibility matrix;
- provider guides;
- Rusty/CLI reference;
- upgrade guides;
- examples/reference applications;
- troubleshooting;
- "when not to use Atlasyr."

## 12.5 Release channels

Consider:

- stable;
- beta;
- upstream-canary/nightly.

Stable must use qualified Topcoat dependencies.

## 12.6 Support policy

Define:

- SemVer expectations;
- MSRV policy;
- Topcoat compatibility window;
- deprecation period;
- security support window;
- release cadence based on need rather than arbitrary dates.

## 12.7 Governance

Before broader community adoption:

- contribution guide;
- code of conduct if appropriate;
- security reporting process;
- architecture decision record process;
- release authority;
- maintainer expectations.

### Milestone 12 exit criteria

- Atlasyr can be consumed as a product rather than copied as an internal experiment;
- users can understand supported versions and upgrade paths;
- public branding and attribution are legally/operationally ready.

---

# Continuous engineering tracks

These do not wait for one milestone.

## Topcoat release intake

- watch upstream releases and relevant `main` changes;
- maintain capability ledger;
- run canary;
- remove Atlasyr duplication when appropriate.

## Dependency and supply-chain health

- keep Rust and frontend graphs audited;
- update vulnerable/yanked dependencies;
- maintain source/integrity rules;
- preserve locked/reproducible builds.

## Technical debt

- eliminate the tracked Clippy legacy baseline over time;
- do not allow the baseline to grow;
- keep dead code and compatibility shims temporary;
- delete superseded POC code.

## Documentation freshness

- current version claims must distinguish current state from historical evidence;
- no old "planned" language after implementation;
- capability ledger and compatibility matrix updated with each Topcoat adoption.

## Performance discipline

- benchmark only questions that matter;
- compare against previous Atlasyr versions before broad marketing claims;
- use controlled workloads;
- distinguish framework overhead from DB/network/application work;
- never claim a generic Rust performance victory from anecdotal POC results.

## Security review

- threat-model new cross-cutting capabilities;
- preserve negative tests;
- review trust boundaries whenever deployment topology changes.

## Upstream contribution

- send generic fixes/improvements upstream when they fit Topcoat;
- avoid building a permanent private fork by accumulation.

---

# Product success measures

Atlasyr should measure success in ways that reflect its actual thesis.

## Developer velocity

First establish a measured baseline, then track:

- manual registration/wiring steps per feature;
- non-domain boilerplate;
- number of files touched for common slices;
- feedback-loop latency;
- clarity of compiler/startup errors;
- generator refusal/rollback reliability;
- frontend-only changes that avoid Rust rebuilds;
- time required to integrate an existing Rust crate/service.

## Upstream adaptability

Track:

- time/effort required to qualify a new Topcoat release;
- number of compatibility breaks detected by canary before release adoption;
- Atlasyr code deleted after upstream replacement;
- number of permanent compatibility shims;
- capability-ledger freshness.

## Production quality

Track:

- release-gate reliability;
- security regression coverage;
- dependency-audit health;
- startup/readiness failure clarity;
- clean graceful shutdown;
- reproducibility of production artifacts.

## Framework focus

Atlasyr is succeeding when a Rust-heavy team can expose and operate its Rust capabilities through a secure web application **without adopting a second backend runtime merely to obtain web-framework ergonomics**.

---

# Initial priority queue

Unless new evidence changes the order, the immediate queue is:

1. finalize README/ROADMAP product transition;
2. record and freeze the qualification decision;
3. resolve the arbitrary trusted-proxy network cap;
4. revalidate Topcoat 0.10 development reload / `dev.js`;
5. create the first capability ledger;
6. add Topcoat attribution/NOTICE;
7. implement Topcoat compatibility workflow and canary;
8. choose the Development Velocity v1 reference feature;
9. measure the current feature workflow;
10. simplify core framework ceremony;
11. expand Rusty based on the measured repetition;
12. re-measure velocity;
13. begin broader web/data/async integration tracks only after those foundations are green.

The roadmap may reorder later milestones as Topcoat evolves or real applications expose stronger priorities. Milestone 0, Milestone 1, and the measured Development Velocity work are intentionally first because they determine whether Atlasyr can remain maintainable while both Atlasyr and Topcoat continue to move.
