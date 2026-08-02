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
- **Description**: Inspect the plugin assembly for restricted namespaces, types, members, assemblies, and unmanaged modules, and validate momentary action behavior.
- **Verification**: `PluginSecurityValidation` exits 0.
- **Covers**: REQ-015, REQ-016, REQ-018, REQ-020
- **Status**: Automated

## TEST-012
- **Title**: Hub VAR packaging validation
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
- **Verification**: CodeQL completes without an unresolved blocking finding.
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
- **Description**: On a disposable copy of a representative AddonPackages library, run check, deep check, plan, one applied repair or migration, report review, and checksum-verified restore.
- **Verification**: VaM loads the restored library after a fresh rescan and the evidence record includes paths, hashes, reports, and observed result.
- **Covers**: REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-019, REQ-022
- **Status**: Manual required for beta release

## TEST-019
- **Title**: Installer and uninstall smoke test
- **Type**: e2e
- **Description**: Install as a standard Windows user, reject an unsafe backup path, confirm automatic engine startup and plugin installation, then uninstall and confirm backups/reports remain.
- **Verification**: The procedure completes without elevation and records retained user data and removed application components.
- **Covers**: REQ-002, REQ-017, REQ-022
- **Status**: Manual required for beta release

## TEST-020
- **Title**: Live VaM plugin workflow
- **Type**: e2e
- **Description**: In VaM 1.22.0.12, load the Session Plugin, run check/plan and one backed-up mutation, observe busy-state locking and terminal status, and confirm an automatic package rescan.
- **Verification**: VaM remains stable, the engine reports match the requested operation, and the post-operation rescan reflects the result.
- **Covers**: REQ-015, REQ-016, REQ-022
- **Status**: Manual required for beta release

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
- **Description**: Review user and maintainer documents plus installer/release text for canonical VaMender naming, beta posture, supported platform, backup warnings, limitations, and independent-project disclosure. Confirm the README provides distinct, implementation-accurate Session Plugin and standalone CLI procedures, including read-only planning, mutation gates, backup/report locations, rescans, and restore.
- **Verification**: No conflicting production/GA claim or omitted safety warning remains, and both documented usage routes can be followed without relying on unstated mutation or restore behavior.
- **Covers**: REQ-001, REQ-002, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-021, REQ-022, REQ-023
- **Status**: Manual required for beta release
