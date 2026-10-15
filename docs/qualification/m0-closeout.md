# Atlasyr Milestone 0 — Corrective Closeout Evidence

## Baseline

The Atlasyr product transition was integrated into `main` by
pull request #1 at commit:

`b4dbd835346a1fdac880e09fd181d840762217e0`

The original qualification history was preserved. POC 6's
Adopt With Constraints decision, scorecard, and qualification
tags remain part of the repository record.

The product's pinned Topcoat baseline is 0.10.0.

## Corrective work

### Trusted-proxy network configuration

The inherited fixed limit of 64 trusted proxy networks had
no established threat-model or resource justification.

The arbitrary count restriction was removed from
`src/config/proxy.rs`.

Strict address and CIDR validation, catch-all rejection,
and fail-closed behavior remain in place.

The focused proxy regression suite passed all eight tests
on 2026-10-15.

### Topcoat 0.10 development broadcaster

The Docker development-endpoint issue originally observed
against Topcoat 0.9 was reproduced against pinned 0.10.0.

The Topcoat broadcaster served `/dev.js` from within the
application container but was unreachable from the host
because it listened only on container loopback.

This is a development-only compatibility limitation.
A custom Atlasyr networking workaround was deferred.

Detailed record:
`docs/qualification/topcoat-0.10-dev-endpoint.md`

### Capability ownership

The initial capability ledger records the 22 capabilities
required by `ROADMAP.md`, including current integration,
upstream overlap, security obligations, ownership, and
reassessment triggers.

Ledger: `docs/capability-ledger.md`

### Attribution

The project provides a standalone `NOTICE` and preserves
the pinned upstream Topcoat MIT license text at:

`third_party/licenses/Topcoat-MIT.txt`

Atlasyr remains independent from Topcoat and Tokio.

## Qualification and release boundaries

- Historical POC tags remain unchanged.
- The full canonical local `./release-gate` is mandatory
  before merging the corrective branch into `main`.
- A final M0 product-transition checkpoint may be tagged
  after the corrective merge and its verification.
- GitHub Actions is CI verification, not production deployment.
- Public trademark, domain, package-name, and related naming
  clearance must be completed before public product branding.
- The Docker development-broadcaster limitation remains
  documented until upstream changes or a justified adapter
  resolves it.

This record summarizes verification evidence. It does not
replace test logs or claim that public-release requirements
have been satisfied.
