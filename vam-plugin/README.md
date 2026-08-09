<!-- SPDX-License-Identifier: MIT -->

# VaMender in-VaM control panel

VaMender is a scene-independent Session Plugin for Check, Deep Check, Plan,
Repair, Migrate, Full Optimize, Restore Most Recent, and Restore All. VaM stays
open during operations and rescans its package registry when work completes.

![VaMender Session Plugin control panel](../docs/images/interface/04-vam-session-plugin-panel.png)

## Installation

1. Download and run the project-published beta
   `VaMender-Setup-<version>.exe` from
   [VaMender releases](https://github.com/TheAgenticCreator/vamender/releases).
2. Choose the folder containing `VaM.exe` and a durable backup folder outside
   `AddonPackages`. Setup installs the engine and
   `AgenticCreator.VaMender.1.var`; no PowerShell script or open console is
   required.
3. In VaM, add
   `AgenticCreator.VaMender.1:/Custom/Scripts/AgenticCreator/VaMender/VaMender.dll`
   as a **Session Plugin**.
4. For automatic startup, open **Session Plugin Presets**, choose
   **Change User Defaults**, then **Set Current as User Defaults**.

Uninstall from Windows **Installed apps > VaMender**. Uninstalling removes the
per-user engine and its startup registration, but deliberately retains backups,
reports, and the installed VAR so user data is never silently deleted.

GitHub Releases is the sole binary source for VaMender. An F95Zone discussion
post may point to the matching GitHub release, but it is not an alternate
download host. VaMender is not distributed through VaM Hub because its engine
modifies VARs in `AddonPackages`.

VaMender does not require a scene or atom. Open its panel from Session Plugin
Custom UI. On VaM's default scene it also places a themed **Open VaMender**
button beside **Open Default Scene**. VaMender does not modify the Add-On
Package Manager or duplicate VaM's Hub and package-update controls.

VaM does not expose a supported API for third-party plugins to register a new
top-level main-menu tab. The Session Plugins tab is therefore VaMender's native
home. VaM's VaMX bootstrap is hard-coded for VaMX; VaMender does not impersonate
VaMX or patch VaM.

## What runs where

The Session Plugin owns the VaM-native window, user actions, status display,
and package rescan. The installed VaMender engine performs the actual
whole-library work: dependency/reference analysis, VaM-log correlation,
checksum-backed filename/archive/metadata repair, safe dependency relinking,
dependency-closure cleanup, old-version migration, verification, and restore.
Every changed or archived VAR is copied to the selected backup folder first.

VaM intentionally prevents ordinary `MVRScript` plugins from writing
`AddonPackages`, launching a process, loading a native library, or referencing
the unrestricted `MVR.FileManagement` API. The plugin therefore exchanges a
fixed allowlist of operation requests and status records with its private
per-user engine through VaM's permitted `Saves/PluginData` storage. The user
never starts that component manually; VaMender Setup installs and starts it.

This split is deliberate. Reimplementing archive repair in C# would still not
grant a normal plugin authority to replace or archive VARs, while bypassing the
sandbox with junctions or an injected mod loader would weaken VaM's security
model. VaMender keeps the tested engine outside the sandbox and the UI inside
VaM.

## Compatibility and license

The plugin was tested with VaM `1.22.0.13`, the current verified runtime. VaM
`1.22.0.12` is expected to work because the plugin uses stable CLR 2,
Session Plugin, secure-file, and Unity UI surfaces, but that version has not
been directly tested. See [`docs/VAM-COMPATIBILITY.md`](../docs/VAM-COMPATIBILITY.md)
for the exact impact surface and limits.

The VaM VAR is licensed `CC BY 4.0` and identifies **AgenticCreator** with the
project repository as its attribution link. The source code is also available
under MIT.

Contributors can build the DLL with:

```powershell
.\tools\build-vam-plugin.ps1 -VaMPath "D:\VaM"
```

The build targets VaM's CLR 2 profile and rejects a DLL unless runtime type
loading, `MVRScript` subtype discovery, and VaM 1.22.0.13 sandbox metadata
validation all pass.

The committed DLL is the hash-locked CLR 2 baseline built and type-load
validated against VaM 1.22.0.13. For a release containing no native behavior
change, GitHub Actions verifies the baseline DLL and normalized source-tree
hashes, then stamps only equal-length version, beta URL, and Setup wording
strings before repeating sandbox metadata validation and VAR packaging. Any
other C# or DLL change fails closed and requires a fresh native build using the
licensed VaM installation; VaM assemblies are never committed or published.

Never use VaMender restore points as your only backup. Keep a separate, tested
copy of important data.

VaMender remains beta until the evidence gates in
[`docs/ROADMAP.md`](../docs/ROADMAP.md) are satisfied and the maintainer
explicitly approves v1.0.0 promotion.
