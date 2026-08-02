<!-- SPDX-License-Identifier: MIT -->

# VaM Hub Resource Draft — VaMender

This is a copy-ready draft plus a compliance checklist. It is not legal advice.
VaM Hub policies can change; recheck the linked official pages immediately
before publishing.

## Policy review and current decision

- **Category**: `Plugins + Scripts` is correct. The official category guide says
  it covers in-game plugins and some external applications, with external apps
  commonly receiving a moderator-added risk warning:
  <https://hub.virtamate.com/threads/04-category-descriptions.67784/>.
- **Free hosting**: the primary VaM resource must be uploaded to the Hub, not
  replaced with an external download:
  <https://hub.virtamate.com/threads/06-posting-guide-for-free-resources.67610/>
  and <https://hub.virtamate.com/threads/all-free-resources-must-be-hosted-on-the-hub.60972/>.
- **VAR contents**: the release VAR contains only VaMender's `meta.json` and
  original precompiled plugin DLL. It contains no nested VAR/archive,
  dependency, third-party content, or Windows executable. This matches the free
  resource packaging rules.
- **License**: the Hub VAR is `CC BY 4.0`, credits AgenticCreator, and has no
  dependencies. Repository source remains MIT. As the sole creator, the project
  may distribute its own compiled VAR under CC BY while offering source under
  MIT. See the official license guide:
  <https://hub.virtamate.com/wiki/license_help/>.
- **Installer/engine**: this needs moderator pre-review. Current policy allows
  external apps in Plugins + Scripts, and current approved resources use
  sandboxed VAR clients with external daemons, but Hub staff have also raised
  security concerns about hosted EXEs and tools that rewrite many VAR hashes.
  Keep the resource a draft until staff confirm whether the Setup executable
  may be attached, or whether only the Hub VAR should be attached with the
  repository in the Promotional Link field.
- **External links**: do not put direct installer/ZIP download links in the
  description. Use the form's Promotional Link and Alternative Support URL.
- **Images**: use a real screenshot of the VaMender Session Plugin UI in the
  description. The icon may be a branded thumbnail. Do not use AI-generated or
  AI-upscaled Hub screenshots. See the free posting guide above.
- **Automation**: VaMender never accesses, scrapes, uploads to, or automates the
  VaM Hub. GitHub support handoff is user-initiated and sends no diagnostics
  automatically.

## Moderator support-ticket questions before publishing

Send these questions with a link to the draft resource and source repository:

1. May a Hub-hosted, sandboxed Session Plugin VAR require VaMender's open-source
   per-user Windows companion engine for filesystem repair operations?
2. Should `VaMender-Setup-<version>.exe` be attached/scanned as a supplementary
   Plugins + Scripts file, packaged in a ZIP, or distributed only through the
   Promotional Link?
3. Are the warnings about local hash changes, backups, and never redistributing
   modified third-party VARs sufficient for a backup-first repair tool?
4. Is `CC BY 4.0` for the Hub VAR plus MIT for repository source acceptable as
   disclosed in `meta.json` and the resource description?

## Resource form values

### Category

**Plugins + Scripts**

### Title

**VaMender**

### Tag line

**Backup-first VAR repair and dependency cleanup with a sandboxed in-VaM control panel.**

### Promotional Link

`https://github.com/TheAgenticCreator/vamender`

This is the project website/source link, not a direct asset URL.

### Type

**Upload — I will host my resource on the Hub.**

### File Attachments

Attach first:

`AgenticCreator.VaMender.2.var`

Use the exact VAR produced by the approved GitHub Actions tag build. Do not
attach a locally rebuilt VAR. Do not attach Setup/ZIP/EXE until moderators
answer the questions above.

### Version number

**First governed Hub beta: `0.1.1`**

Publish that value only after Cargo, installer, changelog, tag, and CI artifacts
all use the exact same version. Otherwise enter the exact version printed by
the GitHub Actions release artifact.

The Hub resource version is the VaMender product SemVer. The attached VAR uses
VaM's separate monotonically increasing integer revision, so this release is
product `0.1.1` packaged as `AgenticCreator.VaMender.2.var`.

### VAR Health Report

Upload the exact CI-built `AgenticCreator.VaMender.2.var`. Expected facts:

- Package ID: `AgenticCreator.VaMender.2`
- License: `CC BY`
- VaM program version: `1.22.0.12`
- Dependencies: none
- Contents: `meta.json` and
  `Custom/Scripts/AgenticCreator/VaMender/VaMender.dll`
- No embedded executable, source archive, nested VAR, or third-party content

Stop and investigate every Health Report warning before publishing.

### Description

Copy the following text into the resource description, then add at least one
real screenshot of the VaMender Session Plugin UI.

---

**VaMender — production-grade safety, currently beta**

VaMender is a backup-first repair and dependency-cleanup tool for Virt-a-Mate
1.22.0.12 on Windows x64. It evaluates the complete installed AddonPackages
graph together with VaM's package-rescan evidence, then plans or applies only
changes it can justify.

**Important safety warning**

Never run any cleanup tool against your only copy of AddonPackages. Keep a
separate, tested full backup on reliable storage. VaMender creates and verifies
per-VAR restore points before every rewrite or archive, but those restore points
are not a substitute for your own independent backup.

VaMender changes local package bytes and hashes when it repairs a VAR. This can
affect Hub/version identification. Preserve original packages, never upload or
redistribute a modified third-party VAR, and restore/re-download the creator's
original before sharing or diagnosing it elsewhere.

**What the attached VAR contains**

The Hub-hosted VAR contains the sandboxed VaM Session Plugin control panel only:
VaMender's `meta.json` and precompiled CLR 2 plugin DLL. It contains no Windows
executable, third-party content, or dependencies.

VaM's plugin sandbox does not permit an ordinary plugin to rewrite
AddonPackages. Repair operations therefore use VaMender's open-source per-user
Windows companion engine. The plugin communicates through VaM's permitted
Saves/PluginData folder using a fixed operation allowlist; plugin requests
cannot override the library, backup, or report roots selected during setup.
Use the Promotional Link above for source, release checksums, setup, and full
documentation.

**Capabilities**

- Read-only inventory and optional full CRC validation
- VaM-log-aware repair and dependency-closure planning
- Conservative metadata, casing, filename, and supported ZIP-header repairs
- Exact-version preservation for scripts and plugins
- Safe local relinking for proven compatible non-plugin content
- Verified whole-VAR backup before every rewrite or archive
- Checksum-verified selective or complete restore
- Predictable actions-taken, actions-required, and missing-dependency reports
- In-VaM Check, Deep Check, Plan, Repair, Migrate, Full Optimize, and Restore

**Privacy-safe support reports**

VaMender can create a local diagnostic ZIP containing only extracted
package-related issues. It does not include VAR payloads, complete VaM logs,
absolute paths, manifests, credentials, or private URLs, and it never uploads
anything automatically. Package names can still be sensitive, so users must
review every file before manually attaching it to GitHub support.

**Limits and rights**

VaMender does not download missing content, bypass paid/private access, infer an
unknown license, invent replacement assets, or repair arbitrary scripts and
artistic content. Users remain responsible for package licenses, authorized
access, backups, and reviewing every plan.

**Compatibility**

- Virt-a-Mate 1.22.0.12
- Windows x64
- Session Plugin; no scene or atom required
- Current release channel: beta

**License and independence**

The Hub VAR is CC BY 4.0 and should be attributed to AgenticCreator. Repository
source is available under MIT. VaMender is an independent community project and
is not affiliated with or endorsed by Meshed VR or Virt-a-Mate.

For setup, source, checksums, roadmap, and issue reporting, use the Promotional
Link and Alternative Support URL on this resource.

---

### Description images

Recommended gallery, using real application output and synthetic package data:

1. A 16:9 hero screenshot showing the VaMender Session Plugin Custom UI in
   VaM's default or empty scene.
2. The Check or Plan result with package counts and the read-only/no-changes
   state visible.
3. The backup-first repair confirmation showing a sample backup destination and
   planned actions.
4. A completed restore/status view.
5. The real Windows Setup page for selecting VaM and backup folders, using
   non-personal sample paths.
6. The local support bundle and `README_FIRST.txt` review warning, using only
   synthetic package identifiers.
7. Optionally, the themed **Open VaMender** default-scene button.

Capture at approximately 1920×1080 and crop for readable UI. Use the existing
thumbnail only if its provenance is non-AI and the project owns it. Do not show
real library inventories, private package names, third-party/adult content, or
credentials. Do not use AI-generated imagery or AI upscaling. Simple titles,
arrows, borders, captions, and branding around genuine screenshots are allowed;
do not alter what the actual VaM or VaMender UI depicts.

### Search Results Image Gallery

**Enable Image Gallery**

### Credits

- **Creator Name**: `AgenticCreator`
- **Support Link or Link to Resource**:
  `https://github.com/TheAgenticCreator/vamender`

No third-party package/content credits are needed for the VAR itself because it
contains only original VaMender content and declares no dependencies. Add linked
credits for any non-built-in content visible in screenshots.

### Tags

`dependency, var, repair, cleanup, backup, restore, package manager, session plugin, utility, windows, beta`

### Additional information URL

`https://github.com/TheAgenticCreator/vamender#readme`

### Alternative support URL

`https://github.com/TheAgenticCreator/vamender/issues`

### Icon

Upload `assets/vamender-hub-icon-100.png`.

The image is 100×100 and intended for the Hub search/resource thumbnail.

### Policy agreement

Check the agreement only after reviewing the live policies and terms on the day
of submission:

- <https://hub.virtamate.com/forums/policies.39/>
- <https://hub.virtamate.com/help/terms/>

### Scheduled Release Options

Choose **Save as draft** first. Post immediately only after:

- the moderator questions are resolved;
- the exact CI VAR has a clean Health Report;
- the real UI screenshot and thumbnail are present;
- all URLs, credits, license text, and beta version match the artifact; and
- the release tag's GitHub Actions build/package/upload jobs are green.
