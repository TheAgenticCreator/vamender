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

## 2026-07-31 — README usage routes and initial beta release plan

- **Author**: VaMender
- **Type**: user documentation and evidence mapping
- **REQs affected**: REQ-014, REQ-015, REQ-016, REQ-017, REQ-023
- **Changes**: Expanded the README into separate VaM Session Plugin and standalone CLI routes. Documented plugin loading, online/offline status, every operation's mutation behavior, report and backup locations, automatic rescans, restore behavior, and the initial `v0.1.0` beta release plan.
- **Evidence**: Compared instructions with the CLI argument definitions, bridge dispatch, Session Plugin action wiring, architecture, and requirement records. Expanded TEST-022 to review both routes explicitly; no GitHub release or tag existed during the reset to the initial release sequence.
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
- **Evidence**: Formatting, strict Clippy, and all 20 Rust tests pass. Inno Setup 6.7.3 compiled the corrected Setup. A live running-host upgrade completed in approximately two seconds with Restart Manager reporting no file users; installed executable and initial revision-1 VAR hashes matched their build artifacts; the obsolete pre-release plugin artifact was copied to durable `install-history` before removal; exactly one restarted engine owned the lock and advanced `heartbeat.txt`; Start with Windows remained registered. A synthetic RUNNING state made `stop-host` fail closed without terminating the engine; idle shutdown then exited cooperatively, removed the lock, preserved startup registration, and no-argument launch restored exactly one healthy engine.
- **Status**: local supported-Windows installer regression passed; fresh CI/CodeQL and remaining disposable-library/VaM acceptance evidence are required; release prohibited

## 2026-08-02 — pull-request identity gate correction

- **Author**: TheAgenticCreator
- **Type**: CI identity diagnosis and privacy hardening
- **REQs affected**: REQ-020, REQ-027
- **Root cause**: Pull-request checkout targets GitHub's temporary merge commit by default. The first PR run therefore rejected GitHub-generated author/committer metadata before any build step even though both project-authored commits passed the local canonical identity checker.
- **Changes**: Both Windows CI jobs now validate `github.event.pull_request.head.sha` for pull requests and `HEAD` for push/dispatch events. The failed run containing the temporary merge metadata must be deleted after replacement checks are registered.
- **Evidence required**: YAML parse, simulated pull-request and push revision selection, identity validation of the real branch history, green replacement Windows CI/CodeQL, and failed-run deletion verification.
- **Status**: correction implemented; replacement CI and privacy cleanup pending; release prohibited

## 2026-08-02 — installer, native UI, and disposable-library acceptance

- **Author**: TheAgenticCreator
- **Type**: supported-Windows acceptance evidence and documentation
- **REQs affected**: REQ-001, REQ-007, REQ-012, REQ-017, REQ-020, REQ-021, REQ-022, REQ-028
- **Changes**: Added tightly cropped real tray and in-VaM control-panel screenshots, visibly redacted personal filesystem paths, and rendered before/after images sourced directly from an isolated synthetic library's VaMender reports.
- **Evidence**: The final Setup upgraded a running idle tray host without Restart Manager intervention, matched executable and plugin build hashes, preserved startup registration, and restarted exactly one lock-owning host with an advancing heartbeat. The installed tray launched VaM; the main-menu Open VaMender control opened the native Session Plugin UI without loading a scene. A three-VAR disposable library produced one safe exact-version relink and one missing-dependency quarantine plan; automatic cleanup checksum-backed up affected packages, rewrote both metadata and content references, archived the broken dependent, and completed with an empty missing-dependency report. Full manifest restore then recovered both the original reference and archived scene while preserving overwritten files under restore-conflicts. VaM was closed normally, all raw captures were deleted, and no live library VAR was changed for the demonstration.
- **Status**: local installer, tray, native UI, disposable cleanup, and restore acceptance passed; PR CI/CodeQL must pass after documentation changes; release prohibited

## 2026-08-04 — Zoo Code project configuration and AI-router profiles

- **Author**: TheAgenticCreator
- **Type**: maintainer tooling, agent safety, and documentation configuration
- **REQs affected**: REQ-001, REQ-018, REQ-020, REQ-021, REQ-022, REQ-023, REQ-027
- **Changes**: Added project-scoped Zoo Code governance rules, mode-specific operating rules, safe slash commands, a secret-excluding `.rooignore`, a tightened workspace command policy, and a sanitized five-profile import template for the local/cloud LiteLLM router. Updated the project Specsmith default model to `qwen38-4b-distilled` without changing product runtime behavior or storing cloud credentials.
- **Evidence required**: JSON parsing, focused configuration inspection, Specsmith audit, and confirmation that cloud credentials remain outside the repository.
- **Status**: configuration implemented; product release posture unchanged

## 2026-08-04 — Zoo Code mode profile completion

- **Author**: TheAgenticCreator
- **Type**: maintainer tooling and agent configuration correction
- **REQs affected**: REQ-018, REQ-020, REQ-022, REQ-023
- **Changes**: Completed the Zoo Code import template with explicit LiteLLM assignments for Ask, Architect, Code, Debug, Orchestrator, and a project Review mode. Added advanced output/reasoning limits per mode and enabled prompt caching only for the cache-capable Qwen cloud profiles.
- **Evidence**: Profile and mode JSON validation passed; local profiles remain cache-disabled and cloud profiles remain cache-enabled.
- **Status**: configuration complete; product release posture unchanged

## 2026-08-04 — Zoo Code configuration storage correction

- **Author**: TheAgenticCreator
- **Type**: maintainer tooling configuration and documentation correction
- **REQs affected**: REQ-018, REQ-020, REQ-023
- **Changes**: Separated the project Review mode into the supported `.roomodes` workspace file and clarified that provider profiles live in Zoo Code's VS Code Secret Storage, are imported from the Settings page, and are associated with modes from the Prompts tab. Removed custom-mode data from the provider profile import template.
- **Evidence**: `.roomodes` and provider profile JSON parse successfully; no cloud credential is stored in the repository.
- **Status**: configuration corrected; product release posture unchanged

## 2026-08-04 — Release media kit and VaM plugin documentation captures

- **Author**: TheAgenticCreator
- **Type**: release documentation, Hub preparation, and privacy-safe media
- **REQs affected**: REQ-001, REQ-018, REQ-022, REQ-023, REQ-025
- **Changes**: Added a clean 16:9 VaM Session Plugin hero and a tight panel crop derived from the genuine committed UI capture, added license sidecars, wired the crops into the README, installation manual, plugin README, and Hub draft, and added a copy-ready release media kit with gallery order, captions, provenance, privacy constraints, and remaining optional live-capture guidance.
- **Evidence**: New images were visually inspected, dimensions and local references were validated, and no new live VaM installation or private scene content was introduced.
- **Status**: documentation/media package prepared; beta release and live acceptance gates remain governed by CI and manual evidence

## 2026-08-04 — Disposable release scenario suite and VaM installation check

- **Author**: TheAgenticCreator
- **Type**: beta acceptance evidence, release documentation, and privacy-safe screenshots
- **REQs affected**: REQ-003, REQ-004, REQ-005, REQ-006, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-018, REQ-019, REQ-022, REQ-024
- **Changes**: Added `tools/run-release-scenarios.ps1`, documented seven disposable-library scenarios, and committed seven sanitized report captures covering clean/deep check, missing dependencies, metadata repair, corrupt archives, migration/restore, broken-library quarantine, and privacy-safe support reporting.
- **Evidence**: The installed `vamender 0.1.0` engine completed all ten scenario invocations with exit code 0. Mutation cases created backup manifests before changes; migration restored two VARs from its manifest; the support bundle excluded the full log, absolute paths, and package inventory by default. VaM launched successfully from the supplied installation and its genuine default-scene `Open VaMender` launcher was captured; automated activation of the native panel was not accepted as complete, so the existing approved genuine panel capture remains the documentation asset.
- **Status**: disposable CLI evidence passed; live VaM plugin workflow and final release CI remain required; release prohibited

## 2026-08-04 — Absolute VAR package output path correction

- **Author**: TheAgenticCreator
- **Type**: release tooling defect correction
- **REQs affected**: REQ-020, REQ-021, REQ-022
- **Changes**: Updated `tools/package-vam-plugin.ps1` to resolve absolute `-OutputPath` values without prefixing the repository root.
- **Evidence**: The package gate now succeeds with an absolute temporary output path and emits a metadata-validated `AgenticCreator.VaMender.1.var` with a SHA-256 digest.
- **Status**: local packaging regression passed; fresh CI release evidence remains required

## 2026-08-05 — Live VaM Session Plugin check capture

- **Author**: TheAgenticCreator
- **Type**: live VaM integration evidence and release media
- **REQs affected**: REQ-002, REQ-015, REQ-016, REQ-022, REQ-023
- **Changes**: Opened the installed Session Plugin through the default VaM scene's `Open VaMender` button, captured the native panel, and ran the read-only `Check Library` action through the bridge. Added a path-redacted live-check panel capture for internal documentation.
- **Evidence**: The panel reported `VAMENDER — ENGINE ONLINE`, the check entered RUNNING state, then completed successfully with bridge report `639215229734409946`. The installed VaM reports version 1.22.0.13, so this confirms integration behavior but does not replace the supported 1.22.0.12 beta acceptance gate. No live mutation was run.
- **Status**: live read-only plugin check passed; backed-up mutation/rescan and supported-version acceptance remain required

## 2026-08-05 — VaM compatibility evidence basis correction

- **Author**: TheAgenticCreator
- **Type**: compatibility policy and release documentation correction
- **REQs affected**: REQ-002, REQ-016, REQ-020, REQ-022, REQ-023, REQ-025
- **Changes**: Changed the verified runtime claim to VaM 1.22.0.13, updated plugin metadata and packaging validation to match the actually tested installation, and documented the concrete CLR 2, Session Plugin, secure-file, and Unity UI impact surface. VaM 1.22.0.12 is now explicitly described as expected but untested.
- **Evidence**: The live panel and read-only check passed on VaM 1.22.0.13. No .12 run is claimed; compatibility language now distinguishes direct evidence from engineering inference.
- **Status**: compatibility documentation aligned with observed evidence; beta mutation/rescan acceptance remains required

## 2026-08-05 — Hub gallery review and screenshot privacy cleanup

- **Author**: TheAgenticCreator
- **Type**: release media review, Hub draft update, and privacy cleanup
- **REQs affected**: REQ-018, REQ-022, REQ-023, REQ-025
- **Changes**: Reviewed all 23 committed image assets, added the fresh redacted VaM 1.22.0.13 live-check panel to the Hub gallery, and replaced local-user paths in the two installer screenshots with neutral sample paths.
- **Evidence**: All media was visually inspected. The VaM bridge was complete and VaM closed normally. Hub and media-kit image references resolve; the gallery contains only project-owned, synthetic, redacted, or genuine application UI captures.
- **Status**: media and Hub draft ready for formal release review; exact CI VAR, current Hub policy check, moderator questions, and beta release gates remain pending

## 2026-08-05 — CI-equivalent installer and live beta gates

- **Author**: TheAgenticCreator
- **Type**: pre-release installer regression, health report, and live VaM acceptance
- **REQs affected**: REQ-002, REQ-003, REQ-009, REQ-010, REQ-011, REQ-012, REQ-015, REQ-016, REQ-017, REQ-020, REQ-021, REQ-022, REQ-023
- **Changes**: Built the current release-equivalent Windows engine, `.13` plugin, Hub VAR, and Inno Setup bundle from the worktree; installed the Setup into the supplied VaM 1.22.0.13 installation using its independent backup; ran installer/uninstaller/reinstall regression; generated a real read-only library check and privacy-first support bundle; and ran a disposable VaM-mounted migration through the live Session Plugin.
- **Evidence**: Current VAR SHA-256 `f4d3707dfa7df6ef7a8eecbe778b8eeeae4a9e24533a906dbefb7b41ef61414b`; Setup SHA-256 `54201419007fea748aaf6facaaab028a27b48976d4df4d357c4fd6018e26790e`. The real library reported 3,926 VARs, 0 invalid archives, 0 missing dependencies, and 0 unresolved package IDs. Setup installed matching plugin bytes, preserved the prior VAR in `install-history`, uninstall removed engine/config while retaining plugin, backup, and reports, and reinstall restored the running host. Live bridge request `639215297283257255` rewrote one disposable scene, archived one older VAR after backup, and VaM logged `Scanned 3 packages`, `Package changes detected`, and `COMPLETE — VaM completed its native AddonPackages rescan`; the original 3,926-VAR library was restored and verified.
- **Status**: local beta gates passed; exact current tagged GitHub Actions VAR/Setup, final tag validation, actual Hub upload/health report, and moderator policy answers remain release prerequisites
