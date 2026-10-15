# Topcoat 0.10.0 — Docker development endpoint verification

Status: Confirmed upstream development-networking limitation.
Verified: 2026-10-15.
Atlasyr baseline: Topcoat 0.10.0.

## Background

The Topcoat 0.9 qualification identified an inaccessible browser
development-script endpoint when running inside Docker.

The original observation was historical, not evidence of the
behavior of Topcoat 0.10.0. This record documents revalidation.

## Verified observations

- Atlasyr `/` returned HTTP 200.
- Atlasyr `/login` returned HTTP 200.
- The login HTML emitted a Topcoat development-script URL.
- The observed URL was `http://127.0.0.1:59039/dev.js`.
- The development broadcaster was listening on `127.0.0.1`
  inside the application container.
- An internal request to `/dev.js` returned HTTP 200 and
  12,392 bytes.
- The same URL was unreachable from the host browser environment.
- Docker Compose published the application port 3000, not the
  dynamically selected development broadcaster port.

Port 59039 is an observed example, not a guaranteed fixed port.

## Diagnosis

Topcoat's development broadcaster is operational inside Docker,
but its loopback-only listener cannot be reached directly from
the host browser. Publishing a Docker port alone does not resolve
an internal loopback-only binding.

This is a development-networking integration limitation, not
evidence of an application HTTP or production runtime failure.

## Decision

Defer a custom Atlasyr networking adapter.

Rationale:

- Avoid duplicating Topcoat's private development protocol.
- Avoid a brittle workaround for a dynamically assigned port.
- Keep production runtime behavior independent of this issue.
- Reassess when upstream changes its development server or when
  developer-experience measurements justify a narrow adapter.

This decision does not assert that browser hot reload works
through the current Docker configuration.

## Future compatibility trigger

Reassess when Topcoat offers a configurable development listener,
a supported proxy mechanism, or different Docker development
behavior. Record the outcome in the Atlasyr capability ledger
and Topcoat compatibility review.
