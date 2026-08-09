<!-- SPDX-License-Identifier: MIT -->

# VaMender GitHub Release and F95Zone Media Kit

This checklist supplies assets for the beta manual, GitHub release, and F95Zone
announcement. GitHub Releases is the only binary distribution channel; F95Zone
may host the discussion and images but must link to the exact tagged GitHub
release instead of attaching a separate executable, ZIP, or VAR.

## Copy-ready documents

- GitHub release and F95Zone announcement: `docs/F95ZONE-RELEASE.md`
- Illustrated Windows installation manual: `docs/INSTALLATION.md`
- In-VaM plugin instructions and architecture boundary: `vam-plugin/README.md`
- Full user manual, CLI route, safety model, and reports: `README.md`

## F95Zone image order

Use these images in this order in the F95Zone post. The first two are genuine
VaM Session Plugin output; the report images are genuine VaMender output from
an isolated synthetic library; the installer images are genuine Setup output.

| Order | Asset | Caption / alt text | Dimensions |
| --- | --- | --- | --- |
| 1 | `docs/images/interface/03-vam-session-plugin-hero.png` | VaMender Session Plugin control panel with the engine online | 542×681 |
| 2 | `docs/images/interface/05-vam-session-plugin-live-check.png` | VaM 1.22.0.13 read-only Check completed; local paths redacted | 542×681 |
| 3 | `docs/images/installer/03-vam-folder.png` | Setup selects the VaM folder using a sanitized sample path | 597×462 |
| 4 | `docs/images/installer/04-backup-folder.png` | Setup selects durable backup storage using a sanitized sample path | 597×462 |
| 5 | `docs/images/demo/01-before-plan.png` | Read-only cleanup plan identifies a relink and unresolved dependency | 1600×900 |
| 6 | `docs/images/demo/02-after-cleanup.png` | Backup-first cleanup completes and the missing-dependency report is empty | 1600×900 |

If F95Zone offers a thread avatar, cover, or thumbnail field, use the
project-owned `assets/vamender-icon-100.png`. It is a brand marker, not a
substitute for the first genuine Session Plugin screenshot. Do not attach the
standalone VAR or another binary to the post: normal users install the Setup
executable from the matching GitHub release.

## Manual image placements

- `docs/INSTALLATION.md` uses installer captures, the tray menu, and the clean
  panel crop.
- `README.md` uses the tray menu, clean panel crop, and two report captures.
- `vam-plugin/README.md` uses the clean panel crop beside installation and the
  sandbox explanation.
- `docs/RELEASE-SCENARIOS.md` uses the fresh redacted live-check panel capture;
  it records the verified VaM 1.22.0.13 runtime and the untested-but-expected
  1.22.0.12 compatibility case.

## Validation evidence gallery

These synthetic report captures document the release scenario suite and are
appropriate for the manual, GitHub release notes, or F95Zone post. They do not
replace a genuine VaM UI screenshot:

- `docs/images/scenarios/01-clean-inventory.png`
- `docs/images/scenarios/02-missing-dependency-plan.png`
- `docs/images/scenarios/03-metadata-repair.png`
- `docs/images/scenarios/04-corrupt-archive.png`
- `docs/images/scenarios/05-migration-restore.png`
- `docs/images/scenarios/06-broken-library-run.png`
- `docs/images/scenarios/07-support-report.png`

## Provenance and privacy

- The online and live-check crops are unscaled crops of genuine VaM 1.22.0.13
  captures. They omit unrelated scene pixels and use opaque redaction only over
  local filesystem paths; no UI content was generated, upscaled, or altered.
- Do not post real package inventories, private paths, credentials, third-party
  content, or adult scene content.
- Do not use AI-generated or AI-upscaled screenshots as VaMender evidence.
- Scenario captures are generated from actual VaMender report results using
  synthetic disposable VARs; they contain no live package names or paths.

## Pre-post gate

Immediately before publishing, manually review the current F95Zone rules and
complete TEST-025. Confirm that the post uses the matching tagged GitHub
release link, Setup is the primary installation path, all required beta/safety
disclosures are present, and every image follows the provenance and privacy
rules above. Do not publish or resubmit VaMender to VaM Hub; its moderation
policy prohibits the product's `AddonPackages` mutation model.
