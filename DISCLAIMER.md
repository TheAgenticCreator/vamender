<!-- SPDX-License-Identifier: MIT -->

# VaMender disclaimer

VaMender modifies and archives third-party Virt-a-Mate package files. Although
the project is designed to be conservative, backup-first, and restorable, no
automated tool can understand every creator's intent, every scene, every
script, or every private package arrangement.

## Back up first

Do not use VaMender against the only copy of your `AddonPackages` library.
Before any write-capable operation:

1. Make a complete independent backup of `AddonPackages`.
2. Store it outside the live VaM directory, preferably on another drive.
3. Confirm the destination has enough free space.
4. Keep VaMender's backup directory and `manifest.jsonl` together.
5. Test restoration with non-critical data before relying on it.

VaMender's SHA-256-verified per-VAR backups are an additional recovery layer.
They are not a replacement for a complete external backup, versioned storage,
or a disaster-recovery plan.

## Installed engine and in-VaM companion

The VaM plugin VAR does not contain or launch `vamender.exe`. The
project-published VaMender Setup installs the Session Plugin and a private
per-user engine in one wizard. No PowerShell script, administrator rights,
terminal window, or manual background process is required.

The control panel submits a fixed set of operations through VaM's permitted
secure plugin-data path. The installed engine performs the package work with
the same backup and verification rules as direct CLI use. Write-capable
operations can modify the library selected during Setup while VaM remains
open; the plugin requests a VaM package rescan afterward.

Download Setup only from the project repository's release page and verify its
published checksum. The portable standalone executable remains available for
advanced CLI use. Neither form replaces a complete independent backup.

Repairing a VAR changes its bytes and checksum and can affect Hub/version
identification. Preserve original packages. Never upload or redistribute a
modified third-party VAR; restore or reacquire the creator's original before
sharing it or requesting creator support.

Support bundles can contain package identifiers that reveal paid, private, or
otherwise sensitive content. Review every generated file before sharing it.
VaMender never uploads a support bundle or submits an issue automatically.

## Assumption of risk

You are solely responsible for:

- deciding whether a proposed repair, relink, migration, or quarantine is
  appropriate for your library;
- retaining authorized access to paid, private, deleted, or creator-hosted
  packages;
- complying with package licenses, creator terms, and applicable law;
- reviewing generated reports and validating the result in VaM; and
- restoring or replacing content if an automated change is unsuitable.

Do not use VaMender to bypass authentication, payment, creator restrictions,
licenses, or access controls. Do not distribute backed-up or modified packages
unless you have the right to do so.

## No warranty or liability

VaMender is provided "as is", without warranty of any kind. To the maximum
extent permitted by law, VaMender contributors, maintainers, and the VaMender
project are not liable for any claim, damage, or
other liability, including data loss, file corruption, loss of access to
content, broken scenes or plugins, system instability, downtime, lost revenue,
licensing disputes, privacy incidents, or direct, indirect, incidental,
special, exemplary, or consequential damages arising from use, inability to
use, or misuse of the software.

The terms of the [MIT License](LICENSE) also apply.

## Independence

VaMender is an independent community project. It is not affiliated with,
approved by, sponsored by, or endorsed by Meshed VR, Virt-a-Mate,
SharpVaMTools, the Virt-a-Mate Hub, or third-party package creators.
