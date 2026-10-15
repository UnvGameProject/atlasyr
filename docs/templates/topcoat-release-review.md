# Topcoat Release Review — TEMPLATE

Copy this document into a candidate-specific review
record. Do not edit the shared template for each release.

## Candidate identification

- Review date:
- Reviewer:
- Atlasyr baseline commit:
- Source Topcoat version:
- Candidate Topcoat version:
- Candidate upstream tag/commit:
- CLI version:
- Review branch:
- Upstream release notes:
- Upstream changelog/source:
- Initial decision: Pending

## Required review evidence

Complete all 16 steps in
`../topcoat-release-intake.md`.

| Step | Review | Result | Evidence / finding |
| --- | --- | --- | --- |
| 1 | Release notes | Pending | |
| 2 | Source and API comparison | Pending | |
| 3 | Breaking changes | Pending | |
| 4 | CLI/runtime coupling | Pending | |
| 5 | Rust/MSRV | Pending | |
| 6 | Security-sensitive behavior | Pending | |
| 7 | Routing | Pending | |
| 8 | Browser/runtime/assets | Pending | |
| 9 | Integrations | Pending | |
| 10 | Atlasyr overlap | Pending | |
| 11 | Compatibility tests | Pending | |
| 12 | Ownership classification | Pending | |
| 13 | Capability ledger | Pending | |
| 14 | Migration/simplification | Pending | |
| 15 | Full release gate | Pending | |
| 16 | Qualified-version decision | Pending | |

Use Pass, Fail, Deferred or Not applicable, with
an explicit explanation for deferred or N/A items.

## Affected Atlasyr contracts

Record affected routes, sessions, cookies, middleware,
renderers, htmx, assets, runtime startup, shutdown
and other relevant boundaries.

List new or modified tests and their results.

## Capability ownership decisions

For each overlapping capability:

- Capability:
- Upstream implementation:
- Atlasyr implementation:
- Decision: adopt / adapt / coexist / defer / Atlasyr-specific
- Security consequences:
- Code simplification or migration:
- Ledger update:

Repeat this section as needed.

## Migration and rollback

- Source and target dependency pins:
- Lockfile changes:
- Rust/toolchain changes:
- SQLx/PostgreSQL changes:
- Database or configuration migrations:
- Breaking changes visible to applications:
- Required migration instructions:
- Rollback strategy:
- Known limitations:

## Gate and qualification result

- Exact tested commit:
- `./release-gate` result:
- Contract-test evidence:
- Remaining failures:
- Final decision: Pending
- Qualified Atlasyr version:
- Qualified Topcoat version:
- Compatibility-matrix update:
- Reviewer approval:

Do not mark a candidate supported unless the
required review and release gate have passed.
