<!-- DEPRECATED: This file is no longer the source of truth.
     Edit docs/requirements/*.yml (or docs/tests/*.yml) instead.
     This file will be removed after all projects have migrated.
     Run: specsmith migrate run  to update your project. -->
<!-- SPDX-License-Identifier: MIT -->

# VaMender Test Specification

This document maps accepted requirements to automated or explicitly manual
evidence. A test is not considered passed merely because it is listed here.
Release evidence must record the command or procedure, date, environment, and
result. Environment-dependent manual tests remain mandatory for beta release.

## Automated repository gates

## TEST-001
- **Title**: Rust format gate
- **Type**: build
- **Description**: Run `cargo fmt --check` with the pinned Rust toolchain.
- **Verification**: Command exits 0.
- **Covers**: REQ-020
- **Status**: Automated

## TEST-002
- **Title**: Rust lint gate
- **Type**: build
- **Description**: Run `cargo clippy --all-targets --all-features --locked -- -D warnings -D clippy::cognitive_complexity`.
- **Verification**: Command exits 0 with no denied diagnostics.
- **Covers**: REQ-020
- **Status**: Automated

## TEST-003
- **Title**: Rust engine test suite
- **Type**: integration
- **Description**: Run `cargo test --all-targets --all-features --locked`.
- **Verification**: All unit and integration tests pass.
- **Covers**: REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-015, REQ-019, REQ-020
- **Status**: Automated

## TEST-004
- **Title**: Backup and metadata repair integration
- **Type**: integration
- **Description**: Exercise missing metadata reconstruction with an explicit license and verify the whole-VAR backup and manifest record before replacement.
- **Verification**: Rust test `explicit_license_rebuilds_missing_metadata_with_whole_var_backup` passes.
- **Covers**: REQ-005, REQ-010, REQ-011
- **Status**: Automated

## TEST-005
- **Title**: Relink and migration safety integration
- **Type**: integration
- **Description**: Exercise exact-reference fallback, non-plugin relinking, payload conflict filtering, rewrite, old-version archive, and restore.
- **Verification**: The migration and dependency-closure Rust integration tests pass.
- **Covers**: REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012
- **Status**: Automated

## TEST-006
- **Title**: Filename repair safety integration
- **Type**: integration
- **Description**: Exercise unambiguous download-suffix repair and byte-identical malformed duplicate archiving.
- **Verification**: Both filename repair Rust tests pass and retain backup evidence.
- **Covers**: REQ-006, REQ-010, REQ-019
- **Status**: Automated

## TEST-007
- **Title**: Missing member and VaM log diagnosis
- **Type**: integration
- **Description**: Distinguish installed providers with missing internal members and parse VaM missing-clothing warnings.
- **Verification**: The resource-member and VaM-warning Rust tests pass.
- **Covers**: REQ-004, REQ-008, REQ-019
- **Status**: Automated

## TEST-008
- **Title**: Engine bridge request validation
- **Type**: unit
- **Description**: Reject traversal-capable request IDs and accept bounded numeric plugin request IDs.
- **Verification**: Bridge request validation Rust tests pass.
- **Covers**: REQ-015
- **Status**: Automated

## TEST-009
- **Title**: Host installation path safety
- **Type**: unit
- **Description**: Validate configured VaM roots, quote startup paths, reject backups inside AddonPackages, and preserve an existing plugin during installation.
- **Verification**: All `host_install` Rust tests pass.
- **Covers**: REQ-002, REQ-017
- **Status**: Automated

## TEST-010
- **Title**: Plugin compile and CLR type-load validation
- **Type**: build
- **Description**: Restore and build `VaMender.Plugin.Validation.csproj`, then load the packaged plugin against VaM CLR 2 API stubs.
- **Verification**: .NET build and `VaMTypeLoadValidation` exit 0.
- **Covers**: REQ-002, REQ-016, REQ-020
- **Status**: Automated

## TEST-011
- **Title**: Plugin sandbox security validation
- **Type**: integration
- **Description**: Inspect the plugin assembly for restricted namespaces, types, members, assemblies, and unmanaged modules; validate momentary action behavior; and assert repeated unchanged status polling does not write duplicate VaM log entries.
- **Verification**: `PluginSecurityValidation` exits 0.
- **Covers**: REQ-015, REQ-016, REQ-018, REQ-020
- **Status**: Automated

## TEST-012
- **Title**: Session Plugin VAR packaging validation
- **Type**: build
- **Description**: Package `AgenticCreator.VaMender.2.var` and validate required metadata and payload layout.
- **Verification**: `tools/package-vam-plugin.ps1` exits 0.
- **Covers**: REQ-001, REQ-016, REQ-020, REQ-021
- **Status**: Automated

## TEST-013
- **Title**: Windows executable branding and release build
- **Type**: build
- **Description**: Build the locked release executable on Windows and verify product name, original filename, version, and icon resources.
- **Verification**: The `windows-release` CI job exits 0.
- **Covers**: REQ-001, REQ-002, REQ-020, REQ-021
- **Status**: Automated

## TEST-014
- **Title**: Setup build and validation
- **Type**: build
- **Description**: Build the Inno Setup bundle and verify the versioned filename and SHA-256 sidecar.
- **Verification**: The Windows Setup CI step exits 0 and emits the expected artifacts.
- **Covers**: REQ-002, REQ-017, REQ-020, REQ-021
- **Status**: Automated

## TEST-015
- **Title**: Release metadata gate
- **Type**: build
- **Description**: Check stable Semantic Version tag syntax, Cargo version equality, and a dated changelog entry before release jobs run.
- **Verification**: The release validation job exits 0.
- **Covers**: REQ-001, REQ-021, REQ-023
- **Status**: Automated

## TEST-016
- **Title**: CodeQL analysis
- **Type**: build
- **Description**: Run GitHub CodeQL analysis for Rust on pushes, pull requests, and the weekly schedule.
- **Verification**: The pinned `github/codeql-action` init/analyze revisions
  resolve to the reviewed `v4.37.6` patch and CodeQL completes without an
  unresolved blocking finding.
- **Covers**: REQ-018, REQ-020
- **Status**: Automated

## TEST-017
- **Title**: Specsmith traceability audit
- **Type**: build
- **Description**: Run `specsmith sync --check`, `specsmith req gaps`, `specsmith req orphans`, and `specsmith audit` after governance changes.
- **Verification**: Machine state is synchronized and no accepted requirement lacks mapped evidence.
- **Covers**: REQ-001, REQ-020, REQ-023
- **Status**: Local automated

## Manual beta acceptance

## TEST-018
- **Title**: Disposable-library CLI smoke test
- **Type**: e2e
- **Description**: Run the source-controlled ten-scenario `tools/run-release-scenarios.ps1` corpus against synthetic disposable libraries: nested/Unicode inventory, fresh/stale/absent VaM logs, metadata repair, BZIP2 and corrupt/unsupported archives, filename collisions, version migration conflicts, dependency closure, mutation/restore safety, bridge protocol containment, and support-report privacy.
- **Verification**: Every asserted scenario passes, read-only hashes are unchanged, mutation cases create checksum-verified backup manifests before changes, and the evidence record retains reports, hashes, restore output, screenshots, and observed result. VaM must load a restored representative library after a fresh rescan during manual beta acceptance.
- **Covers**: REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-019, REQ-022
- **Status**: Automated locally on 2026-08-12; CI runs the corpus on every Windows packaging and tagged-release build

## TEST-019
- **Title**: Installer and uninstall smoke test
- **Type**: e2e
- **Description**: Use `tools/run-isolated-vam-regression.ps1` and `tools/run-isolated-installer-regression.ps1` against the marker-protected temporary VaM copy with redirected `LOCALAPPDATA`; the latter runs the actual Setup and uninstaller while upgrading a seeded plugin revision 1 to revision 2.
- **Verification**: The procedures record GUI-subsystem validation, direct startup registration, matching plugin checksum, revision-1 preservation, revision-2 installation, generated reports, cooperative lock cleanup, retained backup evidence, real Setup uninstall, and restored user state.
- **Covers**: REQ-002, REQ-017, REQ-022
- **Status**: Automated isolated host and actual silent Setup lifecycle passed locally on 2026-08-12; interactive installer UI acceptance remains required for beta release

## TEST-020
- **Title**: Live VaM plugin workflow
- **Type**: e2e
- **Description**: In verified VaM 1.22.0.13, load the Session Plugin, run check/plan and one backed-up mutation, observe busy-state locking and terminal status, and confirm an automatic package rescan. Record 1.22.0.12 as expected but untested based on the plugin impact surface.
- **Verification**: VaM remains stable, the engine reports match the requested operation, and the post-operation rescan reflects the result.
- **Covers**: REQ-015, REQ-016, REQ-022
- **Status**: Manual accepted locally on 2026-08-05 using VaM 1.22.0.13 and a disposable fixture; VaM 1.22.0.12 remains expected but untested

## TEST-021
- **Title**: Rights and no-network review
- **Type**: manual
- **Description**: Review the release diff and representative execution to confirm no content acquisition, authentication bypass, license guessing, or package-content telemetry was introduced.
- **Verification**: Reviewer records the inspected paths and confirms the policy boundary.
- **Covers**: REQ-005, REQ-018, REQ-023
- **Status**: Manual required for beta release

## TEST-022
- **Title**: Documentation and beta-language review
- **Type**: manual
- **Description**: Review user and maintainer documents plus installer/release text for canonical VaMender naming, beta posture, supported platform, backup warnings, limitations, and independent-project disclosure. Confirm the README provides distinct, implementation-accurate Session Plugin and standalone CLI procedures, including the required Windows companion Setup, read-only planning, mutation gates, backup/report locations, rescans, and restore. Confirm GitHub Releases is the sole binary source, the F95Zone announcement links to the matching release after a current-rules review, and media is genuine and path-redacted with a Session Plugin UI capture prominent in the post.
- **Verification**: No conflicting production/GA claim, omitted safety warning, ambiguous Setup prerequisite, VaM Hub publication instruction, separate F95Zone binary, or disguised download/support link remains; both documented usage routes can be followed without relying on unstated mutation or restore behavior; and release media contains no generated, upscaled, private-path, unrelated-scene, or desktop-background content.
- **Covers**: REQ-001, REQ-002, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-021, REQ-022, REQ-023
- **Status**: Manual required for beta release

## TEST-029
- **Title**: Windows repository hygiene policy
- **Type**: build
- **Description**: On Windows, verify tracked files exclude local VaM runtime state, credentials, assistant memory, and build products; verify `.gitignore` excludes representative local paths; and verify `.gitattributes` checks out project text as CRLF while preserving Git hooks as LF and release payloads as binary.
- **Verification**: `git ls-files -ci --exclude-standard` reports no ignored tracked files; `git check-ignore` matches representative local state; and `git check-attr` reports the required Windows text, hook, and binary classifications.
- **Covers**: REQ-020, REQ-023
- **Status**: Local automated

## TEST-028
- **Title**: Windows tray-host lifecycle
- **Type**: integration
- **Description**: Build and install on Windows x64, verify the notification-area
  icon and menu actions, confirm Start with Windows registers the dedicated
  GUI-subsystem host without a console, and verify disabled startup plus a safe
  tray exit.
- **Verification**: `tools/run-isolated-vam-regression.ps1` passes the structural
  lifecycle checks on a temporary VaM copy; the final beta run additionally
  verifies tray interaction and deduplicated in-VaM polling manually.
- **Covers**: REQ-028
- **Status**: Manual required for beta release
