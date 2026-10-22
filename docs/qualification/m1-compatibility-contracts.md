# M1 — Topcoat Compatibility Contract Coverage

Baseline: Atlasyr 0.1.0 / Topcoat =0.10.0
Milestone: 1.4
Status: Contract coverage established; canonical release-gate
qualification is required before integration.

## Contract boundaries and evidence

| Boundary | Evidence | Verification |
| --- | --- | --- |
| Route discovery, success and 404 | `src/app.rs` security-header router tests | SQLx HTTP tests |
| htmx request context and fragment bridge | `src/app/compatibility_contract_tests.rs` | Two request-context variants and explicit false header |
| htmx cannot bypass authentication | `src/app/compatibility_contract_tests.rs` | Real router rejects unauthenticated protected request |
| Cookie attributes and sessions | `src/app.rs`, `src/auth/lifecycle_tests.rs`, `src/auth/session/tests.rs` | Existing login, session and cookie tests |
| Origin policy and body limits | `src/app.rs` | Existing HTTP security tests |
| Trusted proxy trust decisions | `src/config/proxy.rs`, `production-smoke` | Configuration regression and live trusted/untrusted peer checks |
| Error conversion and sanitation | `src/app.rs` | Real backend failure, sanitized HTTP 500, security telemetry |
| Security headers | `src/app.rs` | Successful, unauthorized, missing and internal-error responses |
| External HTML rendering | `src/web/views/renderer.rs`, `src/app/compatibility_contract_tests.rs` | Template safety and fragment response tests |
| Native layouts and asset context | `production-smoke` | Packaged `/login` HTML, CSS and script references |
| Production bundled assets | `production-smoke` | Packaged asset routes return nonempty responses |
| Runtime startup and shutdown | `production-smoke`, `src/app.rs` | Live startup, SIGTERM, expected runtime policy |
| Failure during production startup | `production-smoke` | Missing templates and assets fail closed |

## Important boundaries

The router-level tests intentionally do not load a
filesystem-backed production asset bundle.

Native page rendering is instead exercised against the
actual packaged application inside production smoke.

Topcoat `main` testing remains a separate non-release
canary and does not qualify upstream development code
for stable Atlasyr deployments.

Mail and WebSocket/SSE/WebTransport are not adopted
Atlasyr capabilities. Dedicated application contracts
for those features are deferred until adoption.

A dedicated synthetic request-handler panic contract
is not yet present. The existing suite covers real
application error conversion and production startup
panic sanitization; these are distinct from testing
a handler panic response.

The current contracts should be re-evaluated at every
Topcoat release intake. Relevant ownership and adoption
decisions belong in `docs/capability-ledger.md`.

## Verification rule

Run focused compatibility tests while developing and
the canonical `./release-gate` on the committed source
slice before merging.

Do not infer successful native asset rendering solely
from the router-level tests. The production smoke
result is required for that contract.
