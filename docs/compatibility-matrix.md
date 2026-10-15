# Atlasyr / Topcoat Compatibility Matrix

Status: Initial Milestone 1 compatibility policy
Established: 2026-10-15

## Compatibility contract

Atlasyr and Topcoat have independent versions.

While Topcoat remains pre-1.0, a stable Atlasyr release
must pin an exact, qualified Topcoat version unless
specific compatibility evidence supports a wider range.

An Atlasyr version does not imply support for every
Topcoat version sharing a major or minor number.

The Rust MSRV declaration and the tested toolchain are
separate facts. A declared MSRV is not evidence that
every older patch toolchain has been tested.

PostgreSQL and SQLx entries describe the qualified
application integration baseline, not the full range
of combinations supported by those projects.

## Qualified baseline

| Atlasyr | Topcoat library | Rust MSRV declaration | Tested Rust toolchain | SQLx | PostgreSQL | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 0.1.0 | =0.10.0 | 1.98 | 1.98.1 | =0.9.0 | 18.6-bookworm | M0 engineering baseline qualified |

### Supporting dependency details

- Topcoat crate requirement and resolved version: 0.10.0.
- Production image installs `topcoat-cli` 0.10.0.
- SQLx crate requirement and resolved version: 0.9.0.
- Tokio dependency requirement: 1.51.1.
- Tokio resolved by the application lockfile: 1.53.1.
- Development container Rust image: 1.98.1-bookworm.
- Frontend container baseline: Node 24.21.0-bookworm-slim.

The matrix records versions actually qualified by
Atlasyr's local release process. Dependency requirements
and resolved versions must not be conflated.

### Evidence checkpoint

- M0 corrective source commit:
  `5787eae44cbabcbc07dd632d413cfcce981fe931`
- Qualified `main` merge:
  `14251790cde3b27e8a8329058ea7aed68a8ce310`
- Engineering checkpoint tag:
  `atlasyr-m0-closeout-v1`
- Canonical local `./release-gate`: passed.
- Application tests: 149 passed.
- Rusty CLI tests: 77 passed.
- Production packaging, dependency auditing and
  runtime smoke tests: passed.

This is an internal engineering qualification, not
a declaration of public release availability.

## Current migration notes

No post-M0 Topcoat version migration has yet been
qualified under the Milestone 1 release-intake workflow.

The qualification history includes previous Topcoat
versions, but their historical use is not a current
Atlasyr compatibility guarantee.

A future release-intake record must identify:

- source and target Topcoat versions;
- CLI/runtime coupling changes;
- Rust MSRV and lockfile changes;
- API and behavior changes;
- database or application migrations, if applicable;
- Atlasyr-owned code migrated, retained or deleted;
- known limitations and rollback expectations;
- complete test and release-gate evidence.

Do not add a candidate version to the qualified matrix
until the review and required tests pass.

## Known limitations

### Docker development reload

The Topcoat 0.10 development broadcaster serves
`/dev.js` inside the Atlasyr application container
but listens on container loopback.

The host browser cannot directly access the dynamically
selected development endpoint in the verified Compose
configuration.

The application HTTP and production runtime gates pass,
but host-browser development reload is not qualified.

See `qualification/topcoat-0.10-dev-endpoint.md`.

### Compatibility boundaries

- No Topcoat version other than 0.10.0 is currently
  qualified by this matrix.
- Upstream `main` is not an approved stable dependency.
- A passing canary never automatically expands support.
- A declared Rust MSRV is not a replacement for testing.
- Public naming and distribution clearance are separate
  from technical compatibility.

## Updating this matrix

Follow `topcoat-release-intake.md`.

Each qualified upgrade must produce an attributable
review record, migration notes, updated capability
ownership decisions and canonical release-gate evidence.

Compatibility-only Atlasyr patch or minor releases may
be appropriate when Topcoat changes require them.

Never silently replace an existing qualified baseline
with an unreviewed upstream version.
