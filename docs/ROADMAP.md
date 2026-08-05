<!-- SPDX-License-Identifier: MIT -->

# VaMender Beta Roadmap to v1.0.0

VaMender is beta software with a production-grade safety target. Version
`1.0.0` is a promotion decision backed by evidence, not merely a version bump.
There are no date promises in this roadmap.

## Current beta line

### 0.1.x — governed beta foundation

- Specsmith requirements, tests, architecture, and ledger are authoritative.
- GitHub Actions builds, tests, packages, checksums, and uploads release assets.
- Backup-first mutation, manifest restore, path containment, bridge allowlisting,
  and plugin sandbox validation remain non-negotiable.
- Support reports are generated locally, privacy-reviewed, and never uploaded
  automatically.
- The VaM Hub resource remains a draft until its Health Report is clean and a
  moderator confirms the external companion engine/installer presentation.

### 0.2.x — representative beta evidence

- Run TEST-018 through TEST-022 on multiple independently backed-up,
  representative libraries in Windows x64 and verified VaM 1.22.0.13.
- Record successful fresh-rescan planning, applied repair/migration, report
  review, selective/full restore, reinstall, and uninstall evidence.
- Triage real support bundles and add regression fixtures without accepting or
  redistributing user VARs.
- Define measurable performance and resource ceilings for small, medium, and
  large AddonPackages libraries.

### 0.3.x — release-candidate hardening

- Close all known data-loss, containment, restore, and bridge-authenticity
  blockers.
- Exercise power loss/interruption, locked-file, insufficient-space, malformed
  archive, stale-log, and partial-install recovery paths.
- Complete installer upgrade/downgrade compatibility and preserve user state.
- Complete dependency/license review and software-bill-of-materials evidence.
- Decide and document Authenticode signing for the engine and Setup. A v1.0.0
  release must not imply signature trust that its artifacts do not possess.

## Required v1.0.0 gates

All gates require dated evidence linked from a release checklist.

1. **Safety and recovery** — repeated backup/hash/manifest/restore tests pass;
   no unresolved critical or high-impact data-integrity issue remains.
2. **Supported environment** — clean and upgrade installs, engine startup,
   Session Plugin loading, busy-state behavior, VaM rescan, and uninstall pass
   on supported Windows x64 and verified VaM 1.22.0.13. VaM 1.22.0.12 is an
   expected-but-untested compatibility case, not a completed acceptance claim.
3. **Compatibility** — representative Unicode, nested-library, compression,
   casing, malformed filename, stale-log, and resource-member cases pass.
4. **Security and privacy** — CodeQL and plugin sandbox gates pass; bridge
   threat model is reviewed; support bundles contain no forbidden data; the
   signing and vulnerability-response posture is documented.
5. **VaM Hub compliance** — the exact release VAR passes the Hub Health Report;
   licensing/credits are accurate; moderator guidance for the companion engine
   and installer is recorded; screenshots and description follow current rules.
6. **Support readiness** — issue forms, support-report workflow, privacy
   guidance, triage expectations, and rollback instructions have been tested by
   beta users.
7. **Release reproducibility** — the approved `v1.0.0` tag matches Cargo,
   plugin, installer, and changelog versions; the current integer VAR revision
   is documented; GitHub Actions alone builds, packages, checksums, retains,
   and publishes every release artifact.
8. **Documentation** — README, Hub resource, support, security, disclaimer,
   architecture, requirements, and changelog describe actual behavior and do
   not carry stale beta-only limitations.
9. **Approval** — Specsmith is healthy, all v1.0 requirements have evidence,
   manual blockers are closed, and the maintainer explicitly approves changing
   REQ-001 from beta to general availability.

## What does not change at v1.0.0

The independent-backup requirement, verified backup-before-mutation rule,
unknown-license fail-closed behavior, strict plugin dependency behavior,
content-rights boundary, and prohibition on redistributing modified third-party
VARs remain permanent product constraints.
