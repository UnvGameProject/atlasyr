# Atlasyr / Topcoat Capability Ledger

Status: Initial Milestone 0 baseline
Reviewed: 2026-10-15
Atlasyr baseline: 0.1.0
Topcoat stable dependency: 0.10.0 (exact pin)
Source baseline: M0 corrective branch from `b4dbd835`

## Purpose and interpretation

This ledger records current implementation ownership, upstream overlap,
security responsibility, and the conditions for revisiting each decision.

It is an evidence-based starting point, not a claim that every upstream API
has been exhaustively tested. A locally resolved upstream crate or module
does not, by itself, establish a usable Atlasyr integration.

**Ownership state** uses the six states defined in `ROADMAP.md`:
Topcoat-native, Atlasyr adapter, Atlasyr-owned, Selectable provider,
Experimental, and Deprecated / upstream-replaced. States may change.

**Roadmap status** identifies features explicitly planned in the published
Topcoat 0.10 documentation. "Not listed" means only that the capability
was not an explicit item on that roadmap; it does not imply a permanent
upstream guarantee or absence of lower-level APIs.

Atlasyr must compare a candidate Topcoat release, Topcoat `main`, and
upstream plans before implementing overlapping framework functionality.

## Sources of evidence

- Atlasyr's pinned `Cargo.toml` and `Cargo.lock`.
- Atlasyr implementation and tests, surveyed in M0-C5A.
- Topcoat 0.10.0 resolved crate exports, surveyed in M0-C5A.
- Topcoat 0.10 documentation:
  https://docs.rs/topcoat/0.10.0/topcoat/
- Topcoat upstream roadmap:
  https://github.com/tokio-rs/topcoat
- Atlasyr project architecture and policies in `README.md`.
- Atlasyr requirements and ownership model in `ROADMAP.md`.
- Development endpoint evidence:
  `docs/qualification/topcoat-0.10-dev-endpoint.md`.

An upstream roadmap entry is an intention, not a shipped feature.
Recheck both published releases and `main` during Milestone 1 intake.

## Capability inventory

<!-- LEDGER-ROWS-START -->

| Capability | Current Atlasyr implementation | Topcoat 0.10 baseline | Upstream roadmap status | Security implications | Ownership state | Migration or deprecation trigger |
| --- | --- | --- | --- | --- | --- | --- |
| Routing | Application routes and policy composition in `src/app.rs` and `src/web/pages/contacts.rs`. | Native router, page and layout facilities. | No separate routing replacement listed. | Authorization on matched routes, error handling, request limits. | Topcoat-native | Revalidate when router, matching or middleware contracts change. |
| Request context | `src/auth/request.rs` adds authenticated application context and request-bound policy. | Native request context `Cx` and router request facilities. | No separate context replacement listed. | Client-IP provenance, trusted proxies, sensitive data isolation. | Atlasyr adapter | Simplify if upstream directly covers the application-specific boundary. |
| Cookies | Cookie setup and browser security policy in `src/app.rs`. | `topcoat-cookie` includes cookie jar, signed and private cookie facilities. | Not listed as unfinished. | Secure attributes, SameSite, signing, encryption and origin policy. | Topcoat-native | Revalidate cookie behavior on any upstream security or lifecycle change. |
| Sessions | `src/auth/driver/local_topcoat.rs` and lifecycle tests implement application persistence and policies. | `topcoat-session` provides session state, configuration and token primitives. | Existing primitives; authentication expansion remains planned. | Rotation, expiry, revocation, storage permissions and token secrecy. | Atlasyr adapter | Replace narrow custom glue when upstream offers an equivalent tested lifecycle. |
| Authentication | Application credential verification, Argon2id, login policies and local driver. | Session primitives available; complete authentication solution not assumed. | Authentication explicitly planned. | Password handling, credential errors, throttling, session issuance. | Atlasyr-owned | Evaluate upstream authentication for policy equivalence before adoption. |
| Authorization | Typed roles, permissions and grants under `src/application/authorization/`. | Router/request primitives; Atlasyr role policy is not an upstream contract. | No equivalent policy replacement identified. | Deny-by-default, actor identity and mutation enforcement. | Atlasyr-owned | Reassess if upstream offers a proven authorization abstraction with equivalent semantics. |
| Validation | Domain input policies, Garde, Ammonia and compile-time input-policy macros. | Request extraction primitives; equivalent application validation not established. | Validation explicitly planned. | Untrusted input, HTML rejection, type validation and injection resistance. | Atlasyr-owned | Compare upcoming upstream validation and remove redundant framework plumbing. |
| Runtime rendering | Native Topcoat view/runtime bridge and optional script integration. | `topcoat-view` and `topcoat-runtime` provide rendering and reactive primitives. | More reactivity explicitly planned. | Escaping, template errors, browser event boundaries. | Topcoat-native | Revalidate view/runtime contracts whenever upstream rendering changes. |
| External HTML views | MiniJinja renderer in `src/web/views/renderer.rs` and response bridge in `src/web/views/rendered.rs`. | Native Rust-authored views and responses; no equivalent external MiniJinja integration established. | Rendering and reactivity enhancements planned. | HTML escaping, undefined variables, production debug suppression. | Atlasyr adapter | Remove bridge complexity if upstream supports equivalent external HTML workflows. |
| htmx | HTML fragments and asset integration for contacts and other pages. | `topcoat-htmx` supplies request and response helpers. | No replacement explicitly listed. | CSRF/origin policy, fragment escaping and authorization. | Atlasyr adapter | Prefer upstream helpers as their integration surface expands. |
| Native Topcoat reactivity | Opt-in Topcoat runtime script through `src/web/layouts/app.rs`. | Runtime, procedures, events and shard primitives. | More reactivity explicitly planned. | Server procedure authorization and browser-to-server trust boundary. | Topcoat-native | Review new runtime APIs without forcing HTML-first pages to adopt them. |
| Assets | `src/web/assets.rs`, frontend build, and production packaging. | `topcoat-asset` supports asset declaration, bundling and serving. | Other frontend and image improvements planned. | Asset integrity, public files, caching and build provenance. | Atlasyr adapter | Simplify external build integration when qualified upstream tooling supersedes it. |
| Mail | No application mail implementation identified in M0 inventory. | `topcoat-mail` 0.10.0 exists in the resolved graph; no Atlasyr adoption demonstrated. | Mail capability already present upstream. | Credentials, SMTP transport, recipient handling and message content. | Experimental | Prefer qualified Topcoat mail integration when a real application feature needs mail. |
| Rate limiting | Login throttle using Governor in `src/auth/driver/throttle.rs`. | Router/integration primitives; direct equivalent of application throttle policy unverified. | Easier rate-limiting middleware explicitly planned. | Account and client budgets, spoofed IPs, abuse resistance. | Atlasyr-owned | Adopt or adapt upstream only if existing fail-closed throttle guarantees survive. |
| Background jobs | No implemented background-job subsystem identified. | No adopted Topcoat job subsystem demonstrated. | Background jobs explicitly planned. | Job permissions, retries, idempotency and secret handling. | Experimental | Reassess upstream before introducing a queue or worker abstraction. |
| Uploads/multipart | Request-body limits and documented upload policy; no complete upload feature demonstrated. | Router/body facilities present; multipart behavior needs focused verification. | No complete upload subsystem identified in the surveyed roadmap. | Size limits, MIME/content validation, path safety and storage isolation. | Experimental | Verify Topcoat multipart APIs and real upload requirements before implementation. |
| OpenAPI | No Atlasyr OpenAPI integration identified. | No adopted OpenAPI implementation demonstrated. | OpenAPI explicitly planned. | Schema exposure, endpoint visibility and auth documentation. | Experimental | Evaluate upstream OpenAPI before implementing duplicate schema generation. |
| Localization | No Atlasyr localization implementation identified. | No adopted localization contract demonstrated. | Localization explicitly planned. | Output encoding, locale trust and translation integrity. | Experimental | Prefer upstream support if it meets real application requirements. |
| Realtime transports | Optional browser reactivity; no separate Atlasyr WebSocket, SSE or WebTransport subsystem established. | Runtime/browser integrations available; transport-specific Atlasyr adoption unverified. | WebTransport explicitly planned. | Origin checks, authentication, connection budgets and event authorization. | Experimental | Select transport only for a measured realtime workload after upstream review. |
| Deployment | Compose, Docker production image, packaging, smoke and dependency gates. | Runtime/application startup primitives; Atlasyr owns deployment package. | Deployment documentation explicitly planned. | Non-root execution, immutable filesystem, secrets and supply chain. | Atlasyr-owned | Simplify packaging if stable upstream deployment guidance becomes applicable. |
| Health/shutdown | `src/application/health.rs`, `src/app.rs`, readiness and shutdown contracts. | Runtime startup and server lifecycle primitives. | No full deployment-lifecycle contract identified in roadmap. | Readiness accuracy, bounded shutdown, crash reporting and availability. | Atlasyr adapter | Retire local integration glue when upstream satisfies the same tested contracts. |
| Observability | Typed security/request events and database audit events. | Topcoat runtime boundary; equivalent Atlasyr event/audit policy not established. | No replacement observability policy identified. | Data minimization, redaction, trace integrity and audit access control. | Atlasyr-owned | Integrate upstream hooks where useful without losing operational security policy. |

<!-- LEDGER-ROWS-END -->

## Current upstream-overlap decisions

These decisions apply to the current M0 baseline. They are **not**
approval to start implementing unbuilt features. A feature slice must
revalidate upstream released functionality, upstream `main`, and its roadmap.

- **Adopt upstream:** routing, cookies, native rendering/reactivity and
  existing htmx primitives where they fit.
- **Adapt upstream:** sessions, request context, assets, health/runtime
  integration and external HTML response bridging.
- **Atlasyr-specific implementation justified:** current credential,
  authorization, input-validation and login-throttling policies, subject
  to ongoing upstream parity review.
- **Defer for upstream or real application pressure:** mail adoption,
  background jobs, OpenAPI, localization and advanced realtime transports.
- **Further verification before design:** multipart/upload behavior.
- **No selectable-provider declaration yet:** optional presentation
  approaches do not automatically imply an established provider interface.

No currently qualified capability has been classified as
Deprecated / upstream-replaced. That state remains available for
future release intake decisions.

## Known development-only compatibility exception

Topcoat 0.10.0 emits a working development script from a broadcaster
bound to container loopback. Under the verified Atlasyr Docker setup,
the host browser cannot reach that endpoint directly.

Decision: defer an Atlasyr networking adapter pending upstream support
or demonstrated development-velocity impact.

This does not affect qualified production application HTTP behavior,
but it means Docker-host browser reload cannot be claimed as working.

See `docs/qualification/topcoat-0.10-dev-endpoint.md`.

## Maintenance contract

When evaluating a Topcoat release or beginning an overlapping Atlasyr
feature slice:

1. Review release notes, relevant sources and upstream roadmap.
2. Revalidate the corresponding Atlasyr implementation and tests.
3. Record whether to adopt, adapt, coexist, defer, or justify ownership.
4. Update ownership state, security obligations and migration trigger.
5. Document any feature-specific compatibility limitations.
6. Run the relevant contract tests and full canonical release gate
   before integration into `main`.

This ledger is a baseline and must evolve with verified upstream changes,
not speculative feature availability.
