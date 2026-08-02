<!-- SPDX-License-Identifier: MIT -->

# Change Ledger

## 2026-07-31 — specsmith import
- Imported project: vamender
- Detected type: cli-rust
- Language: rust
- Build system: cargo

## 2026-07-31T11:40 — specsmith preflight accepted utterance "Initialize Specsmith governance and define complete requirements, architecture, tests, verification, and release posture for a production-ready VaMender beta" (work_item_id=WI-CC6AB0507D8A, confidence_target=0.7).
- **Author**: specsmith
- **Type**: preflight
- **REQs affected**: REQ-085
- **Status**: complete
- **Chain hash**: `d59523ba1e4e7c83...`

## 2026-07-31T11:40 — work_proposal WI-CC6AB0507D8A: Initialize Specsmith governance and define complete requirements, architecture, tests, verification, and release posture for a production-ready VaMender beta
- **Author**: specsmith
- **Type**: work_proposal
- **REQs affected**: REQ-044,REQ-085
- **Status**: complete
- **Chain hash**: `a1505bfbb8ecff94...`

## 2026-07-31T12:14 — specsmith preflight accepted utterance "Add a privacy-first VaMender support-report workflow, GitHub issue intake for VAR lists and VaM errors, VaM Hub compliance documentation, and a documented beta-to-v1.0.0 promotion path [REQ-001 REQ-013 REQ-018 REQ-022 REQ-023]" (work_item_id=WI-4E1A848AE917, confidence_target=0.8).
- **Author**: specsmith
- **Type**: preflight
- **REQs affected**: REQ-085,REQ-001,REQ-013,REQ-018,REQ-022,REQ-023
- **Status**: complete
- **Chain hash**: `84805eeaf5afdbe3...`

## 2026-07-31T12:14 — work_proposal WI-4E1A848AE917: Add a privacy-first VaMender support-report workflow, GitHub issue intake for VAR lists and VaM errors, VaM Hub compliance documentation, and a documented beta-to-v1.0.0 promotion path [REQ-001 REQ-013 REQ-018 REQ-022 REQ-023]
- **Author**: specsmith
- **Type**: work_proposal
- **REQs affected**: REQ-044,REQ-085,REQ-001,REQ-013,REQ-018,REQ-022,REQ-023
- **Status**: complete
- **Chain hash**: `6641cc96b6c3c3d2...`

## 2026-07-31 — beta governance, support reporting, Hub review, and CI release authority

- **Author**: Codex for AgenticCreator
- **Type**: implementation and verification
- **Work item**: `WI-4E1A848AE917`
- **REQs affected**: REQ-001, REQ-020, REQ-021, REQ-023, REQ-024, REQ-025, REQ-026
- **Changes**: Added privacy-first local support bundles and consent-gated GitHub handoff; expanded issue forms; defined beta-to-v1.0.0 evidence gates; documented VaM Hub packaging, licensing, listing, and moderator questions; enforced beta prereleases and tagged native plugin builds through GitHub Actions.
- **Evidence**: Rust formatting, YAML/JSON parsing, PowerShell syntax, Hub VAR packaging, SPDX coverage, whitespace validation, Specsmith sync/gaps/orphans/audit, and Specsmith verification passed. Current official GitHub Action majors were checked against their upstream repositories.
- **Deferred release evidence**: Rust compile/test requires the GitHub-hosted toolchain because this secondary computer has no MSVC linker. The CLR 2 plugin release build requires the controlled self-hosted `vam-1.22.0.12` runner and licensed VaM SDK. The exact CI VAR still requires a clean Hub Health Report and moderator confirmation before posting.
- **Status**: implementation complete; beta publication gates remain intentionally open

## 2026-07-31T17:07 — specsmith preflight accepted utterance "Prepare and publish VaMender 0.1.1 as a GitHub prerelease by replacing the unavailable self-hosted plugin build with a hash-locked, equal-length CI release stamp of the validated CLR 2 baseline, then merge PR 1 and verify the release [REQ-001 REQ-020 REQ-021 REQ-023 REQ-026]" (work_item_id=WI-49D242428161, confidence_target=0.8).
- **Author**: specsmith
- **Type**: preflight
- **REQs affected**: REQ-085,REQ-001,REQ-020,REQ-021,REQ-023,REQ-026
- **Status**: complete
- **Chain hash**: `5bcff9bbe9ad0ece...`

## 2026-07-31T17:07 — work_proposal WI-49D242428161: Prepare and publish VaMender 0.1.1 as a GitHub prerelease by replacing the unavailable self-hosted plugin build with a hash-locked, equal-length CI release stamp of the validated CLR 2 baseline, then merge PR 1 and verify the release [REQ-001 REQ-020 REQ-021 REQ-023 REQ-026]
- **Author**: specsmith
- **Type**: work_proposal
- **REQs affected**: REQ-044,REQ-085,REQ-001,REQ-020,REQ-021,REQ-023,REQ-026
- **Status**: complete
- **Chain hash**: `2df1ac7e31ed157e...`

## 2026-07-31 — VaMender 0.1.1 beta release preparation

- **Author**: Codex for AgenticCreator
- **Type**: release preparation and verification
- **Work item**: `WI-49D242428161`
- **REQs affected**: REQ-001, REQ-020, REQ-021, REQ-023, REQ-026
- **Changes**: Set product, CLI, plugin-source, and installer version to `0.1.1`; advanced the independent VaM package revision to `AgenticCreator.VaMender.2`; replaced the unavailable self-hosted release dependency with a fail-closed CI stamp that accepts only the approved equal-length release strings after verifying the CLR 2 baseline and normalized source hashes; added a concrete Hub screenshot plan.
- **Local evidence**: Release stamping produced a DLL containing `0.1.1`, the beta releases URL, and current Setup wording with no stale embedded strings. Revision-2 VAR packaging passed with only `meta.json` and the stamped plugin DLL. Cargo formatting/metadata, YAML/JSON parsing, PowerShell parsing, whitespace checks, and Specsmith sync/coverage/audit passed.
- **Publication evidence required**: PR CI and CodeQL must pass after these changes; merge must complete; the `v0.1.1` tag must produce a GitHub prerelease whose Setup, portable ZIP, and `AgenticCreator.VaMender.2.var` assets and SHA-256 sidecars all succeed.
- **Status**: implementation complete; publication verification pending

## 2026-07-31 — README usage routes and beta release history

- **Author**: VaMender
- **Type**: user documentation and evidence mapping
- **REQs affected**: REQ-014, REQ-015, REQ-016, REQ-017, REQ-023
- **Changes**: Expanded the README into separate VaM Session Plugin and standalone CLI routes. Documented plugin loading, online/offline status, every operation's mutation behavior, report and backup locations, automatic rescans, CLI dry-run and apply gates, manual rescans, restore behavior, and future screenshot insertion points. Replaced autogenerated GitHub release notes with project-authored beta notes and marked both `v0.1.0` and `v0.1.1` as prereleases.
- **Evidence**: Compared instructions with the CLI argument definitions, bridge dispatch, Session Plugin action wiring, architecture, and requirement records. Expanded TEST-022 to review both routes explicitly. Verified both GitHub release records use `(beta)` titles, prerelease status, and VaMender project wording without autogenerated personal attribution.
- **Status**: documentation complete; manual supported-environment walkthrough remains part of beta acceptance

## 2026-08-02 — pseudonymous maintainer identity enforcement

- **Author**: VaMender
- **Type**: governance, privacy, and release hardening
- **REQs affected**: REQ-001, REQ-020, REQ-021, REQ-023, REQ-027
- **Changes**: Defined VaMender as the collective maintainer pseudonym; added repository-local identity setup, pre-push author/committer checks, full-history CI validation, annotated release-tag validation, contributor-credit guidance, and an explicit boundary between Git metadata and GitHub's authenticated event actor.
- **Evidence required**: Identity checker passes on the clean rewritten root and annotated release tag; a synthetic non-VaMender commit is rejected; Specsmith sync, gaps, orphans, and audit pass; the replacement public repository exposes no personal commit metadata or personal authenticated event actor.
- **Status**: implementation prepared locally; publication requires a separate pseudonymous GitHub account or GitHub App credential
## 2026-08-02 — Windows tray-host engine lifecycle

- **Author**: VaMender
- **Type**: Windows integration, lifecycle, and safety
- **REQs affected**: REQ-015, REQ-017, REQ-023, REQ-028
- **Changes**: Replaced the invisible logon bridge process with a notification-area host that runs the same constrained engine; added Launch VaM, report/backup shortcuts, Start with Windows, About/version/safety details, cooperative Exit, a Start-menu shortcut, and a no-argument installed-host restart path. VaM closing intentionally does not terminate the host because the VaM plugin sandbox cannot safely spawn it for a later ordinary launch.
- **Evidence required**: Formatting, Clippy, Rust tests, Windows release build, Specsmith sync/audit, installer compilation, live per-user host install, tray presence, fresh heartbeat/lock, startup registration, one-click restart, and cooperative shutdown verification.
- **Status**: implementation complete; verification pending
## 2026-08-02 — tray-host verification evidence

- **Author**: VaMender
- **Type**: verification
- **REQs affected**: REQ-015, REQ-017, REQ-020, REQ-028
- **Evidence**: Exact CI-equivalent formatting, locked Clippy with cognitive-complexity denial, all 19 locked Rust tests, locked release build, executable branding/version/icon checks, Specsmith sync and 29-check audit passed. The final release executable was installed per-user against the configured VaM and backup roots; exactly one `host` process held the bridge lock, reported READY, advanced its heartbeat, retained Start with Windows registration, and the no-argument restart path exited successfully without creating a duplicate.
- **Limit**: Inno Setup is not installed on this workstation, so Setup compilation remains a GitHub Actions Windows-release gate. The live installed engine executable itself is verified.
- **Status**: implementation and local live verification complete

## 2026-08-02 — TheAgenticCreator publisher identity migration

- **Author**: TheAgenticCreator
- **Type**: governance, privacy, ownership, and repository publication
- **REQs affected**: REQ-001, REQ-020, REQ-021, REQ-023, REQ-027
- **Changes**: Established `TheAgenticCreator <312204356+TheAgenticCreatorDev@users.noreply.github.com>` as the canonical project-authored Git identity and `TheAgenticCreatorDev` as the required GitHub actor. Confirmed the actor is an active owner of the `TheAgenticCreator` organization with administrative access to `TheAgenticCreator/vamender`; updated repository-local defaults, documentation, identity checks, pre-push enforcement, and CI/release expectations. VaMender remains the product name and AgenticCreator remains the VaM package creator ID.
- **Evidence required**: Specsmith sync/audit, synthetic identity rejection, clean parentless commit validation, active-account and repository-permission checks, protected main/default settings, force-with-lease publication, and post-publication history/content/event-actor verification.
- **Status**: implementation in progress

## 2026-08-02 — fresh-repository CI portability correction

- **Author**: TheAgenticCreator
- **Type**: CI diagnosis and pre-release hardening
- **REQs affected**: REQ-017, REQ-020, REQ-021, REQ-027, REQ-028
- **Root cause**: The first clean-root Linux quality job treated Windows-only tray configuration helpers as dead code under `-D warnings`; Windows release, Setup, plugin/VAR packaging, identity, governance, branding, and artifact construction passed.
- **Changes**: Restricted the installed-host configuration and Start-with-Windows helpers to Windows compilation while retaining the non-Windows no-argument fallback. No Windows runtime behavior changed.
- **Evidence**: Local formatting, exact locked Clippy with cognitive-complexity denial, and all 19 locked Rust tests pass. GitHub CI and CodeQL reruns remain mandatory before protection and any initial release.
- **Status**: local fix verified; remote rerun pending; release prohibited

## 2026-08-02 — Windows-only CI and release authority

- **Author**: TheAgenticCreator
- **Type**: platform policy, CI, security analysis, and release governance
- **REQs affected**: REQ-002, REQ-020, REQ-021, REQ-022, REQ-023
- **Changes**: Moved the quality, CodeQL, and release-validation jobs to GitHub-hosted Windows runners; replaced Bash-only SPDX and version/changelog validation with fail-closed PowerShell; made all default run shells PowerShell; renamed required contexts to state Windows scope explicitly. Linux and macOS runners are no longer authoritative or configured because VaMender and VaM support Windows x64 only.
- **Evidence required**: YAML parse, zero Linux/macOS runner or Bash-shell declarations, local PowerShell-equivalent gates, green Windows quality/packaging/CodeQL runs, protected-main required contexts, and no release until the complete Windows/VaM acceptance matrix passes.
- **Status**: implementation complete; verification pending; release prohibited

## 2026-08-02 — installer running-host upgrade regression

- **Author**: TheAgenticCreator
- **Type**: Windows installer safety correction and supported-environment evidence
- **REQs affected**: REQ-017, REQ-020, REQ-022, REQ-023, REQ-028
- **Observed defect**: The exact green-CI Setup artifact reached Windows Restart Manager with the installed VaMender tray engine still holding `vamender.exe`; automatic close timed out after 30 seconds and presented Abort/Retry/Ignore. The first-pass installer otherwise completed after the engine was stopped explicitly.
- **Changes**: Added busy-aware cooperative external shutdown and lock cleanup, a hidden `stop-host` command, an upgrade-compatible pre-install fallback for older engines, checksum verification of the installed Session Plugin, backup-first retirement of older VaMender plugin revisions, and a sanitized seven-screen installation guide.
- **Evidence**: Formatting, strict Clippy, and all 20 Rust tests pass. Inno Setup 6.7.3 compiled the corrected Setup. A live running-host upgrade completed in approximately two seconds with Restart Manager reporting no file users; installed executable and revision-2 VAR hashes matched their build artifacts; revision 1 was copied to durable `install-history` before removal; exactly one restarted engine owned the lock and advanced `heartbeat.txt`; Start with Windows remained registered. A synthetic RUNNING state made `stop-host` fail closed without terminating the engine; idle shutdown then exited cooperatively, removed the lock, preserved startup registration, and no-argument launch restored exactly one healthy engine.
- **Status**: local supported-Windows installer regression passed; fresh CI/CodeQL and remaining disposable-library/VaM acceptance evidence are required; release prohibited

## 2026-08-02 — pull-request identity gate correction

- **Author**: TheAgenticCreator
- **Type**: CI identity diagnosis and privacy hardening
- **REQs affected**: REQ-020, REQ-027
- **Root cause**: Pull-request checkout targets GitHub's temporary merge commit by default. The first PR run therefore rejected GitHub-generated author/committer metadata before any build step even though both project-authored commits passed the local canonical identity checker.
- **Changes**: Both Windows CI jobs now validate `github.event.pull_request.head.sha` for pull requests and `HEAD` for push/dispatch events. The failed run containing the temporary merge metadata must be deleted after replacement checks are registered.
- **Evidence required**: YAML parse, simulated pull-request and push revision selection, identity validation of the real branch history, green replacement Windows CI/CodeQL, and failed-run deletion verification.
- **Status**: correction implemented; replacement CI and privacy cleanup pending; release prohibited
