<!-- SPDX-License-Identifier: MIT -->

# VaMender GitHub Release and F95Zone Announcement

This is the approved distribution and announcement procedure for VaMender
beta releases. It is not legal advice. Review the current F95Zone rules while
signed in immediately before creating or editing a post; do not infer that old
rules, posts, or screenshots remain acceptable.

## Distribution policy

- **Canonical binary source:** the tagged [GitHub Release](https://github.com/TheAgenticCreator/vamender/releases).
  GitHub Actions alone builds the executable and uploads the Setup, portable
  ZIP, Session Plugin VAR, and SHA-256 sidecars.
- **Normal-user installation:** download the versioned `VaMender-Setup-<version>.exe`
  and its `.sha256` sidecar from that GitHub release. Setup installs both the
  per-user engine and bundled `AgenticCreator.VaMender.1.var` Session Plugin.
- **Standalone VAR:** a CI-packaged/manual artifact, not a normal installation
  path and not an F95Zone attachment. It cannot supply the external engine
  needed by the plugin.
- **F95Zone role:** release announcement, screenshots, updates, and discussion.
  It links to the matching GitHub release and must not host a second binary,
  mirror, or a download disguised as a support link.
- **VaM Hub:** do not submit or resubmit VaMender. Its moderators rejected the
  project because the product's third-party engine modifies VARs in
  `AddonPackages`, which their resource policy does not permit. Do not attempt
  to work around that policy through a different resource type or link field.

## Required pre-post review

1. Run TEST-025 and verify the exact tagged GitHub Actions release has all
   assets and SHA-256 sidecars.
2. Open the current F95Zone posting and creator rules while signed in. Apply
   any current rule that is more restrictive than this guide.
3. Use the exact versioned GitHub release URL, not a local file, a mutable
   mirror, or an F95Zone-hosted binary.
4. Confirm the post says **beta**, **Windows x64**, **external engine**, and
   **explicit mutation with verified per-VAR backups**.
5. Confirm it says VaM `1.22.0.13` was tested and VaM `1.22.0.12` is expected
   to work but has not been directly tested.
6. Use only the approved, genuine, privacy-redacted media listed in
   `docs/RELEASE-MEDIA-KIT.md`.

## Copy-ready F95Zone post

Use this text for the initial `v0.1.0` thread. Replace only the versioned
GitHub URL and release-specific changelog summary for later versions.

```markdown
# VaMender v0.1.0 Beta — backup-first VaM VAR diagnosis and repair

**Windows x64 · VaM 1.22.0.13 tested · beta software**

VaMender is a backup-first tool for diagnosing and safely repairing a local
Virt-a-Mate `AddonPackages` library. It combines full-library dependency
analysis with a fresh VaM package-rescan log, makes every mutation explicit,
verifies a whole-VAR backup before changing a package, and records restore
data locally.

## Download and install

Download [VaMender v0.1.0 from GitHub Releases](https://github.com/TheAgenticCreator/vamender/releases/tag/v0.1.0).
Download `VaMender-Setup-0.1.0.exe` and its `.sha256` sidecar from that page,
optionally verify the checksum, then run Setup. Setup installs the external
VaMender engine and the bundled VaM Session Plugin; no PowerShell, administrator
rights, or manual VAR install is required.

## Important safety notice

This is beta software that can modify local VAR files only after you explicitly
choose an applied operation. Keep a separate, tested full backup of your entire
library before use. VaMender's verified per-VAR backups and restore points do
not replace your independent backup. Never redistribute a modified third-party
VAR; restore or reacquire the creator's original first.

## Compatibility

VaMender was tested on Windows x64 with VaM `1.22.0.13`. VaM `1.22.0.12` is
expected to work from the plugin's stable CLR 2/Session Plugin impact surface,
but has not been directly tested. VaMender is an independent community project,
not affiliated with or endorsed by Meshed VR or Virt-a-Mate.

## What it does

- Read-only `Check` and `Plan` operations for package inventory and dependency
  evidence.
- Explicit, backup-first repair, migration, full optimization, and restore.
- A VaM-native Session Plugin panel that requests a package rescan after a
  successful engine operation.
- Local reports and an optional privacy-reviewed GitHub support bundle; no
  package contents or library data are uploaded automatically.

## Screenshots and documentation

Use the screenshots in this post as evidence only: they contain sanitized paths
and synthetic/disposable-library report data. The full installation guide,
safety model, source, and issue tracker are in the GitHub repository:
https://github.com/TheAgenticCreator/vamender
```

## Post maintenance

- Edit the F95Zone post after each tagged GitHub release with that release's
  exact link, checksum guidance, and changelog summary.
- Keep GitHub Releases as the only binary source; never replace the release
  link with a direct asset, cloud mirror, or support-field workaround.
- Treat support reports as private until their owner manually reviews and
  attaches them to a GitHub issue. Never ask users to post unredacted package
  inventories or private paths.
