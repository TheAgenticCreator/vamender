<!-- DEPRECATED: This file is no longer the source of truth.
     Edit docs/requirements/*.yml (or docs/tests/*.yml) instead.
     This file will be removed after all projects have migrated.
     Run: specsmith migrate run  to update your project. -->
<!-- SPDX-License-Identifier: MIT -->

# VaMender Requirements

This document is the canonical product requirement source for Specsmith.
VaMender is the product and project name; `vamender` remains the package,
executable, and CLI command name. The current public release channel is
**beta** even when an artifact is technically suitable for production use.

## Release policy

- **Target quality**: production-grade data safety, integrity, security, and
  release evidence.
- **Current channel**: beta. Documentation, installers, and release notes must
  not describe the current channel as generally available or fully proven.
- **Verified runtime**: 64-bit Windows with Virt-a-Mate 1.22.0.13. VaM
  1.22.0.12 is expected to work based on the documented plugin impact surface,
  but has not been directly tested. Development, unit and integration testing,
  CI, packaging, and release verification are Windows-only; Linux and macOS
  are neither supported product nor validation platforms.
- **Primary user promise**: prove a safe replacement and preserve the original
  before changing a live VAR.

## Product and platform

## REQ-001
- **Title**: Canonical product identity and beta posture
- **Component**: product
- **Status**: Accepted
- **Description**: User-facing surfaces must use the name VaMender. Technical identifiers may use `vamender`. Until a later approved requirement changes the channel, all releases are beta and must carry production-grade safety evidence without claiming general availability.

## REQ-002
- **Title**: Supported platform and host compatibility
- **Component**: distribution
- **Status**: Accepted
- **Description**: VaMender must support 64-bit Windows and the verified Virt-a-Mate 1.22.0.13 runtime, install per-user without administrator rights, and keep development, testing, CI, packaging, and release validation on Windows. Linux and macOS are neither supported product nor validation platforms. VaM 1.22.0.12 remains an expected-but-untested compatibility case based on the plugin impact surface.

## Analysis and planning

## REQ-003
- **Title**: Read-only package inventory
- **Component**: engine
- **Status**: Accepted
- **Description**: `vamender check` must inventory the complete AddonPackages tree without modifying VARs, report invalid archives and unresolved dependencies, and optionally CRC-read every archive member when `--deep` is selected.

## REQ-004
- **Title**: VaM-aware cleanup planning
- **Component**: engine
- **Status**: Accepted
- **Description**: `vamender plan` must combine static archive evidence with a fresh VaM package-rescan log, distinguish stale or absent logs, avoid presenting static-only member references as runtime-confirmed failures, and produce a read-only repair and dependency-closure plan.

## Repair and migration

## REQ-005
- **Title**: Conservative metadata repair
- **Component**: repair
- **Status**: Accepted
- **Description**: VaMender may repair missing, invalid, stale, or incomplete `meta.json` data only from evidence present in the package or supplied explicitly by the user. It must not guess an unknown license or treat descriptive material, morph, or parameter labels as package dependencies.

## REQ-006
- **Title**: Proven filename and archive repair
- **Component**: repair
- **Status**: Accepted
- **Description**: VaMender may correct casing, Unicode-safe identities, supported ZIP header inconsistencies, and malformed filenames only when package identity is unambiguous. Byte-identical malformed duplicates may be archived only after checksum comparison and whole-VAR backup.

## REQ-007
- **Title**: Safe dependency relinking
- **Component**: dependency-closure
- **Status**: Accepted
- **Description**: Exact script and plugin dependencies must remain exact. A reference may be relinked to a newer local version only for non-plugin content when a compatible provider is installed and the relevant resource payload is proven safe.

## REQ-008
- **Title**: Dependency-closure isolation
- **Component**: dependency-closure
- **Status**: Accepted
- **Description**: VaMender must expand failures through the installed dependency graph, differentiate a missing package from a provider missing an internal member, and isolate unusable closures without deleting scenes or silently substituting artistic content.

## REQ-009
- **Title**: Conservative old-version retirement
- **Component**: migration
- **Status**: Accepted
- **Description**: Superseded VAR versions may be archived only after reference rewrites, dependency-closure analysis, payload-compatibility checks, metadata-conflict checks, and post-rewrite validation all succeed.

## Mutation safety and recovery

## REQ-010
- **Title**: Verified backup before mutation
- **Component**: safety
- **Status**: Accepted
- **Description**: Before any VAR is rewritten, renamed, replaced, or archived, VaMender must copy the whole original to a durable backup root outside AddonPackages, verify the copy with SHA-256, and append a restore record to `manifest.jsonl`.

## REQ-011
- **Title**: Atomic replacement and post-change validation
- **Component**: safety
- **Status**: Accepted
- **Description**: Live VAR replacement must use a validated temporary artifact and an atomic or rollback-safe handoff. The resulting archive and affected references must be rechecked before an old version or temporary source is retired.

## REQ-012
- **Title**: Checksum-verified restore
- **Component**: restore
- **Status**: Accepted
- **Description**: Restore must consume VaMender manifest records, verify backup checksums, reject paths outside the selected AddonPackages root, support most-recent subsets, skip existing files by default, and preserve overwritten conflicts under `restore-conflicts`.

## Interfaces and reporting

## REQ-013
- **Title**: Predictable user reports
- **Component**: reporting
- **Status**: Accepted
- **Description**: Every public operation must produce `actions_taken.txt`, `actions_required.txt`, and `missing_dependencies.txt`; the full run must retain detailed stage evidence under `_details` while keeping the three top-level handoff files.

## REQ-014
- **Title**: Explicit CLI mutation gates
- **Component**: cli
- **Status**: Accepted
- **Description**: `check` and `plan` must be read-only. `repair` and `migrate` must default to dry-run and require `--apply` plus a backup root to mutate. `run` must require a backup root. Help output must state the independent-backup warning and user responsibility.

## REQ-015
- **Title**: Constrained engine bridge
- **Component**: bridge
- **Status**: Accepted
- **Description**: The local plugin bridge must bind to installer-selected library, backup, state, and report roots; allow only check, plan, repair, migrate, full run, and restore operations; reject malformed or traversal-capable request IDs; publish heartbeat/status/response state atomically; and prevent concurrent engines for the same state root.

## REQ-016
- **Title**: Sandboxed in-VaM control panel
- **Component**: vam-plugin
- **Status**: Accepted
- **Description**: The CLR 2 Session Plugin must remain inside VaM's sandbox, expose the supported engine operations, prevent competing requests while work is active, display queued/running/terminal/offline state, rescan VaM packages after successful mutation, and avoid direct unrestricted AddonPackages access.

## Installation and lifecycle

## REQ-017
- **Title**: Safe per-user installation and removal
- **Component**: installer
- **Status**: Accepted
- **Description**: Setup must validate `VaM.exe` and AddonPackages, require a durable backup path outside AddonPackages, install the engine and plugin without elevation, configure automatic per-user startup, preserve an existing plugin copy before replacement, and retain user backups and reports on uninstall.

## REQ-018
- **Title**: Content rights and privacy boundary
- **Component**: policy
- **Status**: Accepted
- **Description**: VaMender must not download, authenticate for, bypass access controls on, or infer licenses for paid, private, creator-restricted, or otherwise unavailable content. Local analysis must not transmit package contents or user library data.

## REQ-019
- **Title**: Robust package compatibility
- **Component**: engine
- **Status**: Accepted
- **Description**: Package parsing and reporting must handle Unicode creator/package identifiers, nested AddonPackages paths, supported compressed ZIP members, VaM-sensitive casing, and archives that require the implemented tar-compatible fallback without losing evidence about invalid content.

## Quality, release, and operations

## REQ-020
- **Title**: Required quality and security gates
- **Component**: quality
- **Status**: Accepted
- **Description**: Every merge candidate must pass Rust formatting, SPDX validation, Clippy with warnings and cognitive-complexity findings denied, all locked Rust tests, C# plugin compile/type-load/security validation, Session Plugin VAR packaging validation, Windows release build validation, and CodeQL according to repository CI.

## REQ-021
- **Title**: Reproducible beta release artifacts
- **Component**: release
- **Status**: Accepted
- **Description**: A beta release tag must use Semantic Versioning, match Cargo and installer versions, have a dated changelog entry, build the Windows executable, Session Plugin VAR, Setup executable, and portable ZIP from the tagged commit, and publish SHA-256 sidecars for downloadable artifacts.

## REQ-022
- **Title**: Beta acceptance evidence
- **Component**: release
- **Status**: Accepted
- **Description**: Promotion of a build as a VaMender beta requires green automated gates plus a documented disposable-library smoke test covering install, read-only check/plan, at least one backed-up mutation, VaM rescan, report review, restore, and uninstall. Any unexecuted environment-dependent check must be reported as a release blocker, not silently waived.

## REQ-023
- **Title**: Maintainer and user documentation
- **Component**: documentation
- **Status**: Accepted
- **Description**: README, changelog, support, security, contribution, maintainer, license, and disclaimer files must remain consistent with the VaMender name, beta channel, supported platform, safety model, content-rights boundary, install/restore workflow, and disclosure that the project is independent of Virt-a-Mate.

## Explicit non-goals

VaMender does not repair arbitrary scripts or binary/artistic assets, prove
semantic interchangeability where bytes and metadata cannot establish it,
replace VaM's runtime package rescan, obtain inaccessible content, or eliminate
the user's responsibility to maintain and test an independent full backup.
