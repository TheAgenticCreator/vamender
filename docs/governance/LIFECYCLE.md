<!-- SPDX-License-Identifier: MIT -->

# Project Lifecycle

## Current state: beta

VaMender is feature-complete enough for beta use, but promotion evidence must
still demonstrate the full disposable-library, installer, in-VaM, restore, and
uninstall workflow in the supported Windows/VaM environment.

## Change readiness

- Requirements are accepted and mapped to evidence.
- The implementation is expected to satisfy production-grade safety gates.
- Automated CI and security gates are mandatory for every merge candidate.
- Manual TEST-018 through TEST-022 are mandatory for a beta release candidate.

## Promotion to general availability

GA requires an approved requirement changing REQ-001, every gate in
`docs/ROADMAP.md`, successful repeated beta acceptance on representative
libraries, no unresolved data-loss/security blocker, a GitHub Actions release
as the sole binary distribution source, a current-rule-reviewed F95Zone
announcement that accurately discloses the product, a documented
support/rollback/signing decision, and GitHub Actions release evidence from the
promoted tag. VaMender is not distributed through VaM Hub because its core
workflow modifies VARs in `AddonPackages`.

Run `specsmith checkpoint` for phase readiness and `specsmith audit` for the
governance health gate.
