<!-- SPDX-License-Identifier: MIT -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-02

Initial beta release.

### Added

- Add genuine, privacy-sanitized installer, tray, and in-VaM screenshots plus
  actual before/after reports from an isolated synthetic repair demonstration.
- Add a Windows notification-area host with Launch VaM, report/backup folder
  shortcuts, Start with Windows toggle, version/safety details, and safe Exit.
- Add a Start-menu shortcut and no-argument installed-executable restart path
  so the engine needs no external script or open console.

- Add Specsmith governance with explicit VaMender identity, architecture,
  requirement-to-test traceability, beta acceptance gates, and change ledger.
- Add a privacy-first `support-report` command that extracts package-related
  VaM issues into a local, reviewable ZIP and never uploads it automatically.
- Add dedicated support-report, bug, dependency, and documentation issue forms
  with consent, redaction, package-list, and modified-VAR guidance.
- Add an evidence-based beta-to-v1.0.0 roadmap and a copy-ready VaM Hub resource
  draft with packaging, licensing, screenshot, and moderator-review checks.
- Add a hash-locked CLR 2 release-stamping gate that rejects native plugin
  changes beyond the approved equal-length release metadata strings.

- Inspect complete Virt-a-Mate `AddonPackages` libraries in parallel, with an
  optional full CRC validation pass.
- Reconcile the installed dependency graph with fresh VaM package-rescan and
  error-log evidence instead of recursively downloading into dependency hell.
- Repair missing, invalid, stale, or incomplete `meta.json` dependency data
  from references actually present in each archive.
- Preserve strict script/plugin versions while safely relinking compatible
  non-plugin references to healthy installed providers.
- Repair VaM-sensitive package-name casing, Unicode identifiers, malformed VAR
  filenames, and supported ZIP header inconsistencies when identity is proven.
- Detect installed packages whose referenced internal files are absent and
  distinguish them from genuinely missing package dependencies.
- Plan unusable dependency-closure quarantine without silently deleting scenes,
  substituting artistic content, or bypassing creator access controls.
- Migrate and archive superseded package versions only after dependency-closure,
  payload-compatibility, rewrite, and post-change verification.
- Back up every rewritten or archived VAR first, verify it with SHA-256, record
  it in `manifest.jsonl`, and support selective or complete restore.
- Write three predictable user-facing reports for completed actions, required
  follow-up, and line-separated unresolved dependency IDs.
- Provide concise `check`, `plan`, `repair`, `migrate`, `run`, and `restore`
  commands plus automatic and review-gated PowerShell workflows.
- Provide a one-click per-user Windows Setup with a durable backup-path check,
  automatic engine startup, uninstall support, and no administrator or manual
  PowerShell requirement.
- Provide a precompiled CLR 2 VaM Session Plugin for Check, Deep Check, Plan,
  Repair, Migrate, Full Optimize, and Restore operations while VaM remains open.
- Add a themed **Open VaMender** control beside **Open Default Scene**, while
  leaving VaM's Add-On Package Manager unchanged.
- Rescan VaM's package registry after successful engine operations and report
  live queued, running, waiting, completed, failed, and offline states.
- Lock competing operation controls for the complete engine request lifecycle
  and make every UI control release visually after one activation.
- Include Windows branding, Hub-ready icon assets, a CC BY 4.0 VaM VAR, an MIT
  source license, safety disclaimers, contribution guidance, security policy,
  issue forms, maintainer ownership, Dependabot, CI, CodeQL, and release builds.

### Changed

- Run CI quality gates, CodeQL, release validation, packaging, and artifact
  production exclusively on GitHub-hosted Windows runners.
- Keep the constrained engine available across normal VaM launches and make
  shutdown cooperative so an active backup or repair finishes before exit.

- Treat current VaMender releases as production-grade beta artifacts and make
  GitHub Actions the authority for building, packaging, checksumming, and
  uploading tagged release assets.
- Require version-matched 0.x tags while the beta channel is active and publish
  tagged GitHub releases as prereleases without marking them latest.
- Document that repairs change package hashes, may affect Hub identification,
  and must never be used to redistribute modified third-party VARs.
- Set the initial product version to `0.1.0` and the independent VaM package
  revision to `AgenticCreator.VaMender.1` so both public sequences begin at their
  first release values.

### Fixed

- Stop the running tray engine before Setup replaces its executable, reject an
  upgrade while work is active or queued, and use cooperative shutdown for
  current installations with an upgrade-compatible fallback for older builds.
- Checksum-verify the installed Session Plugin and back up and retire older
  VaMender plugin revisions instead of leaving duplicate revisions installed.
- Validate the real pull-request head in the pseudonymous identity gate instead
  of rejecting GitHub's temporary merge commit.

### Security

- Enforce TheAgenticCreator publisher identity in project-authored commit
  and release-tag metadata with repository-local configuration, pre-push
  validation, CI/release gates, and an explicit `TheAgenticCreatorDev`
  authenticated-actor requirement.

- Keep the VaM plugin inside VaM's sandbox and communicate with the installed
  engine through a fixed operation allowlist in permitted plugin-data storage.
- Bind the engine to the library, backup, and report roots selected during
  Setup; plugin requests cannot override those destinations.
- Reject empty, oversized, non-numeric, and path-traversal engine request IDs
  before constructing report destinations.
- Never acquire paid, private, authenticated, or creator-restricted content and
  never guess an unknown package license.
- Validate the plugin against VaM 1.22.0.12 CLR 2 type loading and namespace,
  assembly, member, and unmanaged-module restrictions before packaging.

[Unreleased]: https://github.com/TheAgenticCreator/vamender/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/TheAgenticCreator/vamender/releases/tag/v0.1.0
