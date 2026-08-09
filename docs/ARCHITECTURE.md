<!-- SPDX-License-Identifier: MIT -->

# VaMender Architecture

## System context

VaMender is a backup-first Windows tool for reconciling a Virt-a-Mate (VaM)
AddonPackages library. Its verified runtime is Windows x64 with VaM 1.22.0.13;
VaM 1.22.0.12 is expected to work but has not been directly tested. The Rust
engine owns archive analysis and every filesystem mutation;
the CLR 2 Session Plugin is a sandboxed UI/client. The current release channel
is beta, with production-grade safety and release evidence required.

## Canonical identifiers

- Product/project: **VaMender**
- Rust crate, binary, and CLI: `vamender`
- Current VaM package revision: `AgenticCreator.VaMender.1`
- Publisher/creator: `AgenticCreator`
- Public maintainer/publisher identity: `TheAgenticCreator <312204356+TheAgenticCreatorDev@users.noreply.github.com>`; GitHub actor `TheAgenticCreatorDev`
- Source repository: `TheAgenticCreator/vamender`

## Components

### CLI and command dispatch

`src/main.rs`, `src/app/cli.rs`, and the dispatcher in `src/app/mod.rs` expose
`check`, `plan`, `repair`, `migrate`, `run`, and `restore`. Hidden `host`,
`install-host`, `stop-host`, `uninstall-host`, and `bridge` commands support the
Windows installer, notification-area lifecycle, and in-VaM integration.
Mutation capability is explicit in the CLI
contract: inspection and planning are read-only, while applied operations
require a durable backup root. See REQ-003, REQ-004, REQ-014, and REQ-017.

### Package analysis engine

`src/app/mod.rs` scans VAR/ZIP content, parses package identities and
references, reads metadata, optionally validates all members, correlates a VaM
log, and produces user reports. `model.rs` contains package, reference, backup,
and VaM-log domain models. `resource_members.rs` identifies installed providers
that lack referenced internal files. See REQ-003, REQ-004, REQ-013, and
REQ-019.

### Repair, dependency closure, and migration

`filename_repair.rs`, `dependency_closure.rs`, and the repair/migration stages
in `mod.rs` calculate conservative changes. Plugin references remain exact;
non-plugin relinks require a compatible installed provider; conflicting
payloads or metadata block retirement. See REQ-005 through REQ-009.

### Mutation safety and restore

Before a live VAR changes, the engine creates and SHA-256 verifies a whole-VAR
backup and appends a `manifest.jsonl` record. Rewrites use temporary artifacts,
archive validation, and controlled replacement. Restore constrains destination
paths, verifies the manifest backup, and preserves displaced conflicts. See
REQ-010 through REQ-012.

### Local engine bridge

`src/app/bridge.rs` monitors a VaM-permitted plugin-data directory. It accepts a
fixed operation allowlist, validates bounded numeric request IDs before forming
report paths, serializes operations with a state lock, and atomically publishes
heartbeat, status, and response files. Installer-selected library and backup
roots cannot be overridden by a plugin request. See REQ-015.

### Windows tray host

`src/app/tray_host.rs` owns the Windows notification-area icon and main-thread
message pump while a large-stack worker runs the same constrained bridge used
by the Session Plugin. Its menu launches VaM, opens reports/backups, toggles the
per-user Run registration, shows compatibility and backup disclaimers, and
requests cooperative shutdown. Closing VaM does not stop this host: VaM's
sandbox cannot reliably spawn an external engine, so the host must already be
available before a normal VaM launch. A no-argument installed executable and a
Start-menu shortcut restart it without an external script. See REQ-028.
### VaM Session Plugin

`vam-plugin/Custom/Scripts/AgenticCreator/VaMender/src` implements a CLR 2
Session Plugin. It builds the VaM-native panel, queues bridge requests, reports
engine state, disables competing operations, launches the panel from VaM's
default-scene UI, and requests a package rescan after successful work. It never
opens or rewrites AddonPackages directly. See REQ-016 and REQ-018.

### Installation and packaging

`installer/VaMender.iss` provides per-user Windows Setup. Before replacement it
rejects active or queued engine work and stops an idle tray host cooperatively;
an older-engine fallback supports upgrades from builds predating `stop-host`.
The hidden host commands checksum-verify the plugin VAR, back up and retire old
VaMender plugin revisions, configure/start the tray-hosted local engine, create
a Start-menu restart shortcut, and remove application components safely.
`tools/package-vam-plugin.ps1` constructs the
Session Plugin VAR; the plugin validation project checks CLR 2 loadability and sandbox
metadata. See REQ-002, REQ-017, and REQ-020.

### Support reporting

`src/app/support_report.rs` performs a read-only library scan and converts
package failures plus recognized VaM package-log evidence into a local review
bundle. It intentionally omits absolute paths, complete logs, archive payloads,
manifests, credentials, and URLs. Full installed-package names require an
explicit flag. A separate confirmation is required before the Windows browser
handoff, and even then no diagnostic content is uploaded automatically. See
REQ-024.

### CI and release automation

`.github/workflows/ci.yml` runs format, SPDX, lint, Rust tests, plugin build and
security validation, Session Plugin VAR packaging, release build, branding, Setup
creation, and CI artifact upload exclusively on GitHub-hosted Windows runners.
`.github/workflows/codeql.yml` performs Rust security analysis on Windows.
`.github/workflows/release.yml` validates a versioned tag and repeats every
release gate on Windows. Linux and macOS runners are intentionally absent
because VaMender and VaM are Windows-only. The CLR 2 plugin is a hash-locked baseline that
was built and type-load validated against VaM 1.22.0.13. For release-only text
changes, CI verifies both that DLL hash and a normalized source-tree hash before
applying three equal-length UTF-16 stamps: product version, beta release URL,
and beta Setup wording. Any other plugin-source or binary change fails closed
and requires a fresh native VaM build and validation. The Windows job packages
the stamped DLL into the current integer-revision VAR, builds the executable,
Setup, and portable ZIP, produces SHA-256 sidecars, and uploads the beta to
GitHub Releases. See REQ-020 through REQ-022.

### Maintainer identity boundary

Public repository history uses the publisher identity `TheAgenticCreator` for
project-authored commits and annotated tags. A repository-local setup tool
configures only this checkout, and a pre-push hook plus CI/release checks reject
unexpected author, committer, or tagger metadata. Contributor credit can be
preserved with `Co-authored-by` trailers. Git commit metadata is distinct from
GitHub's authenticated event actor, so maintainers authenticate as
`TheAgenticCreatorDev` or an approved organization GitHub App.
See REQ-027.

## Primary data flow

1. A user invokes the CLI or an allowlisted plugin action.
2. The engine scans the selected AddonPackages tree and gathers static evidence.
3. Planning optionally correlates a fresh VaM log and builds the dependency
   closure without changing live VARs.
4. An applied operation proves each candidate, creates and verifies a backup,
   writes a validated temporary artifact, replaces or archives the live VAR,
   and rechecks the result.
5. The engine emits the three user handoff reports and detailed stage evidence.
6. Plugin-originated work publishes terminal bridge state and asks VaM to
   rescan its package registry.

## Trust boundaries and invariants

- AddonPackages content and VaM logs are untrusted inputs.
- Bridge request files are untrusted even though they are local.
- Backup and report paths are installer/user-selected but must remain outside
  live package containment where required.
- Package metadata and labels cannot establish a license that the user or
  archive does not explicitly provide.
- The VaM plugin has UI/client authority only; the engine alone owns archive
  mutation.
- GitHub Actions, not a local maintainer build, is the release artifact source.
- Support diagnostics remain local until a user reviews and manually attaches
  them to GitHub; the Hub is never accessed programmatically.
- Modified third-party packages and backups are never distribution artifacts.

## Deployment and state

- Application: `%LOCALAPPDATA%\VaMender`
- Bridge state: `<VaM>\Saves\PluginData\VaMender\Bridge`
- Live content: user-selected `<VaM>\AddonPackages`
- Backups: installer/user-selected durable root outside AddonPackages
- Reports: explicit CLI root or deterministic default; bridge reports under
  the configured state/report root
- Restore index: append-only `manifest.jsonl` in the backup root

## Deliberate non-goals

The engine does not download content, bypass authentication, repair arbitrary
scripts/assets, infer artistic equivalence, or replace VaM's runtime rescan. It
also does not make a per-VAR restore point equivalent to the user's independent
tested full-library backup.
