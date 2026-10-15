# Topcoat Release-Intake Workflow

Status: Milestone 1 §§1.1–1.2
Established: 2026-10-15

## Purpose

Every proposed Topcoat release upgrade must pass
through a deliberate compatibility review.

The procedure applies to candidate upstream releases,
including candidates that ultimately are deferred.

Topcoat `main` canary results inform review but
never authorize a stable dependency upgrade.

## Starting conditions

1. Begin on an isolated compatibility branch from a
   clean, known Atlasyr baseline.
2. Record the existing qualified version, candidate
   version, precise source references and date.
3. Copy `templates/topcoat-release-review.md` into a
   version-specific review record.
4. Retain the old baseline for comparison and rollback.
5. Do not change stable support claims during intake.

A release announcement alone is not sufficient evidence
of API or behavioral compatibility.

## Mandatory 16-step intake

<!-- INTAKE-CHECKLIST-START -->

1. **Release notes:** Retrieve the candidate's tagged release notes and changelog; record authoritative links.
2. **Source/API comparison:** Compare dependency manifests, public APIs Atlasyr consumes, examples and versioned documentation.
3. **Breaking changes:** Identify explicit and implicit source, behavior, feature and configuration incompatibilities.
4. **CLI/runtime coupling:** Check whether `topcoat-cli`, generated assets, runtime modules and tooling require coordinated versions.
5. **Rust/MSRV:** Compare upstream Rust requirements with Atlasyr's declared MSRV, pinned compiler and resolved dependencies.
6. **Security-sensitive behavior:** Inspect sessions, cookies, request context, client-IP handling, authentication boundaries and security advisories.
7. **Routing:** Inspect router registration, extraction, middleware, route matching, errors and response semantics.
8. **Browser/runtime/assets:** Inspect production asset bundling, external scripts, runtime rendering, reactivity and development behavior.
9. **Integrations:** Inspect htmx and other browser or external integrations Atlasyr actually uses.
10. **Overlap discovery:** Compare upstream changes against Atlasyr-owned capabilities and the existing ledger before adding framework code.
11. **Contract tests:** Run or add focused tests for affected behavior, including negative and security cases.
12. **Ownership classification:** Decide adopt, adapt, coexist, defer or Atlasyr-specific, using the ledger's ownership states.
13. **Ledger update:** Record verified upstream capabilities, security responsibilities, ownership changes and reassessment triggers.
14. **Migration/simplification:** Implement required adaptations and remove duplicate Atlasyr code when equivalent upstream behavior is verified.
15. **Release verification:** Run the canonical `./release-gate` against the exact candidate branch and record results.
16. **Support decision:** Approve or defer the candidate, update the compatibility matrix and migration notes only when warranted, and record the precise qualified commit.

<!-- INTAKE-CHECKLIST-END -->

Each step requires a result and supporting evidence.
Not applicable must be explicitly explained rather
than silently omitted.

## Decision rules

### Adopt

Use an upstream capability directly when it satisfies
Atlasyr's contracts without meaningful additional policy.

### Adapt

Keep a narrow, removable seam where Atlasyr requires
application policy, a security boundary, or proven
compatibility insulation.

### Coexist

Retain distinct approaches only when they serve
genuinely different use cases.

### Defer

Retain the current qualified version or implementation
when evidence or application demand is insufficient.

### Atlasyr-specific

Keep ownership of responsibilities that belong to
Atlasyr's application architecture, security policy,
or Rust-heavy integration requirements.

For all outcomes, update the capability ledger where
the decision materially affects ownership or overlap.

## Security and testing requirements

No upstream upgrade is accepted solely because it
compiles.

Review the security-sensitive behavior that Atlasyr
depends upon, including fail-closed configuration,
session lifecycle, trusted-proxy handling, origin
policy, error sanitation and response behavior.

Run affected contract tests and the full release gate
for each proposed release adoption.

A failed check blocks qualification unless its resolution
is documented and verified. Do not reclassify an
unresolved failure as an accepted result without
an explicit, narrowly scoped engineering decision.

## Compatibility records and rollback

A completed intake record must contain:

- previous and candidate dependency versions;
- relevant upstream release and source references;
- local branch, exact commit and lockfile changes;
- the 16 review results;
- new and modified contract tests;
- changes in ownership or capabilities;
- migration actions and deleted redundant code;
- any DB or configuration migration requirements;
- rollback and operational considerations;
- release-gate results and final decision.

Never rewrite a previously qualified matrix entry
to conceal an older supported baseline.

If an upgrade fails, preserve the current pin and
record what blocked the candidate.

## Upstream contributions

A generic Topcoat issue should be evaluated for
upstream reporting rather than permanently worked
around within Atlasyr.

Opening upstream issues, discussions, or pull requests
requires explicit project-owner approval. The intake
process does not authorize automatic external submissions.

## Scope boundaries

Milestone 1.3 will define the separate non-release
Topcoat `main` canary.

Milestone 1.4 will expand focused compatibility
contract tests.

Milestone 1.5 will formalize contribution decisions.

This document establishes the repeatable process for
Milestone 1 §§1.1–1.2; it does not claim those later
engineering deliverables are complete.
