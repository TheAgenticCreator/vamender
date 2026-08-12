<!-- SPDX-License-Identifier: MIT -->

# Isolated VaM Release Testing

Never run release scenarios against the VaM library you use normally. The
release process uses a separate runtime at
`C:\Users\trist\_\VAM\VaMender-ReleaseTest` and creates only synthetic VARs.
The environment is marked with `VAMENDER-ISOLATED-TEST-ENVIRONMENT.txt`; the
setup script refuses to reset an unmarked directory.

## Create or reset the runtime

```powershell
.\tools\new-isolated-vam-test-environment.ps1 -Reset
```

The script copies only the VaM runtime files required to launch VaM. It does
not copy `AddonPackages`, backup libraries, `Custom`, `Saves`, cache, browser
data, downloads, keys, logs, or preferences from the normal installation.

## Run the candidate regression suite

After building the candidate executable and Session Plugin VAR, run:

```powershell
.\tools\run-isolated-vam-regression.ps1 -KeepArtifacts
```

This runs the complete synthetic VAR corpus, temporarily redirects
`LOCALAPPDATA` for the host installation, preserves and restores the existing
Windows startup value, verifies the GUI-subsystem host, sends a real bridge
check through that host, verifies the plugin VAR checksum, and confirms
cooperative shutdown/uninstall. It does not modify the normal VaM library or
the normal VaMender host configuration.

Add `-LaunchVaM` to confirm the disposable runtime stays running. Loading the
Session Plugin panel and observing its status-notification cadence remains a
manual beta-acceptance check in this temporary runtime.

## Exercise the real Setup installer

Exit the normal VaMender tray host first, then run:

```powershell
.\tools\run-isolated-installer-regression.ps1
```

The script runs the actual versioned Setup executable silently with an explicit
temporary application directory, temporary `LOCALAPPDATA`, isolated VaM root,
and isolated backup directory. It upgrades a seeded
`AgenticCreator.VaMender.1.var` to revision `2`, verifies the backup, submits
a bridge check, runs the real uninstaller, and restores the Windows startup
value. It refuses to begin while a normal VaMender host is running.
