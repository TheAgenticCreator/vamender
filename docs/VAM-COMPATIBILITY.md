<!-- SPDX-License-Identifier: MIT -->

# VaM Compatibility Basis

## Evidence status

- **Verified:** VaM `1.22.0.13` on Windows x64. The Session Plugin loaded,
  opened through the default-scene launcher, displayed `ENGINE ONLINE`, and
  completed live read-only `Check Library` plus backed-up migration requests
  through the bridge. The migration used a disposable three-VAR fixture,
  completed VaM's native package rescan, and the real 3,926-VAR library was
  restored before VaM was closed.
- **Expected but untested:** VaM `1.22.0.12`. No direct `.12` acceptance run is
  claimed or required for the current evidence record.

The `.12` expectation is an engineering inference from the plugin's actual
impact surface, not a compatibility test result. Revalidate it if VaM changes
any of the surfaces listed below.

## Plugin-impact surface

The native plugin depends on the following VaM/Unity surfaces:

- VaM CLR 2 loading and `MVRScript` subtype discovery.
- VaM `JSONStorableString`, `JSONStorableAction`, `UIDynamicTextField`, and
  `UIDynamicButton` controls.
- `SuperController.singleton`, its main-menu UI selection, and package-manager
  rescan/open actions.
- `MVR.FileManagementSecure.FileManagerSecure` for the fixed
  `Saves/PluginData/VaMender/Bridge` files.
- Unity `Button` discovery, instantiation, click listeners, and destruction for
  the default-scene launcher.

The plugin does not directly inspect or mutate `AddonPackages`, use private
engine internals, load a scene atom, or depend on version-specific content.
The companion Rust engine owns package operations outside the VaM sandbox.

## Compatibility declaration

Release documentation should say: “Tested with VaM 1.22.0.13. VaM 1.22.0.12
is expected to work because the plugin uses the stable CLR 2, Session Plugin,
secure-file, and Unity UI surfaces listed above, but that version has not been
directly tested.” Do not call `.12` tested or imply a broader VaM compatibility
promise.
