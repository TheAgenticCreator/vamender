<!-- SPDX-License-Identifier: MIT -->

# VaMender Release Media Kit

This checklist gathers the copy and image assets needed for the beta release
manual and the Virt-a-Mate Hub resource. It does not publish to the Hub or
claim that an unrun release gate has passed.

## Copy-ready documents

- Hub resource form and description: `docs/VAM-HUB-RESOURCE.md`
- Illustrated Windows installation manual: `docs/INSTALLATION.md`
- In-VaM plugin instructions and architecture boundary: `vam-plugin/README.md`
- Full user manual, CLI route, safety model, and reports: `README.md`

## Hub gallery

Use the following order for the Hub gallery. The first two are genuine VaM
Session Plugin output; the demo images are genuine VaMender report output from
an isolated synthetic library; the installer images are genuine Setup output.

| Order | Asset | Caption / alt text | Dimensions |
| --- | --- | --- | --- |
| 1 | `docs/images/interface/03-vam-session-plugin-hero.png` | VaMender Session Plugin control panel with the engine online | 1600×900 |
| 2 | `docs/images/interface/04-vam-session-plugin-panel.png` | VaMender native Session Plugin controls and online status | 540×680 |
| 3 | `docs/images/interface/05-vam-session-plugin-live-check.png` | Live VaM 1.22.0.13 read-only Check completed; paths redacted | 540×680 |
| 4 | `docs/images/demo/01-before-plan.png` | Read-only cleanup plan identifies a relink and unresolved dependency | 1600×900 |
| 5 | `docs/images/demo/02-after-cleanup.png` | Backup-first cleanup completes and the missing-dependency report is empty | 1600×900 |
| 6 | `docs/images/installer/03-vam-folder.png` | VaMender Setup selects the VaM folder using a sanitized sample path | 612×471 |
| 7 | `docs/images/installer/04-backup-folder.png` | VaMender Setup selects durable backup storage using a sanitized sample path | 612×471 |

Upload `assets/vamender-hub-icon-100.png` as the Hub resource icon. Use the
exact CI-built `AgenticCreator.VaMender.1.var` for the primary attachment, not
a workstation-built substitute.

## Manual image placements

- `docs/INSTALLATION.md` uses the seven installer captures, the tray menu, and
  the clean panel crop.
- `README.md` uses the tray menu, clean panel crop, and the two report captures.
- `vam-plugin/README.md` uses the clean panel crop beside the installation and
  sandbox explanation.
- `docs/RELEASE-SCENARIOS.md` uses the fresh redacted live-check panel capture;
  it records the verified VaM 1.22.0.13 runtime and the untested-but-expected
  1.22.0.12 compatibility case.

## Validation evidence gallery

These synthetic report captures document the release scenario suite and are
appropriate for the manual or release notes, not as substitutes for a genuine
VaM UI screenshot:

- `docs/images/scenarios/01-clean-inventory.png`
- `docs/images/scenarios/02-missing-dependency-plan.png`
- `docs/images/scenarios/03-metadata-repair.png`
- `docs/images/scenarios/04-corrupt-archive.png`
- `docs/images/scenarios/05-migration-restore.png`
- `docs/images/scenarios/06-broken-library-run.png`
- `docs/images/scenarios/07-support-report.png`

## Provenance and privacy

- The hero and panel crop are lossless crops/compositions of the committed
  genuine VaM UI capture `docs/images/interface/02-vam-control-panel.png`.
- The new hero uses a neutral background only to remove unrelated scene pixels;
  no UI content was generated or altered.
- Existing captures show redacted or synthetic data. Do not add real package
  inventories, private paths, credentials, third-party content, or adult scene
  content to Hub images.
- Do not use AI-generated or AI-upscaled screenshots for the Hub gallery.
- Scenario captures are generated from actual VaMender report results using
  synthetic disposable VARs; they contain no live package names or paths.

## Final capture gate

The current assets satisfy the Hub requirement for a real Session Plugin
screenshot and provide a complete illustrated installation/manual set. If a
fresh VaM acceptance session becomes available before publication, optionally
replace the hero with a current 1920×1080 capture showing the same online state
and capture separate Check, Plan, Restore, and default-scene-button states.
Those states must be recorded as manual beta evidence before being described
as release-proven.
