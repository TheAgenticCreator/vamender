<!-- SPDX-License-Identifier: MIT -->

# Security Policy

## Supported versions

Security updates are provided for the latest released beta version.

| Version | Supported |
| --- | --- |
| Latest 0.x beta | Yes |
| Older or unreleased builds | No |

## Release integrity

Official releases publish SHA-256 sidecars for the Windows Setup executable,
portable ZIP, and self-contained control-panel VAR. Verify the relevant
sidecar before use. A checksum detects unintended change but cannot establish
the authenticity of a file obtained from an untrusted source.

The VAR contains no executable and cannot launch a process. The
project-published Setup installs a private per-user VaMender engine because
VaM's plugin sandbox does not grant normal plugins write access to
`AddonPackages`. The plugin and engine exchange only a fixed operation
allowlist through VaM's secure plugin-data folder. Requests cannot override the
AddonPackages, backup, or report roots selected during Setup.

The optional `support-report` command is read-only and writes diagnostics
locally. It excludes complete VaM logs, absolute paths, VAR payloads, backup
manifests, credentials, and private URLs. It never uploads or submits a GitHub
issue; even its browser handoff requires explicit review confirmation and
leaves attachment and submission to the user.

Any local process running as the same Windows user can potentially alter that
user's plugin-data request files. Install VaMender only for the Windows account
and VaM library you intend to maintain. The engine validates operation names,
uses its installed path configuration, and preserves write targets before
replacement.

## Reporting a vulnerability

Please use the repository's **Security → Report a vulnerability** form. Do
not disclose an unpatched vulnerability in a public issue.

Include the affected version, operating system, a minimal reproduction, the
expected impact, and whether a write-capable command was involved. Remove
personal paths and account information.

Never attach paid/private VARs, authentication material, personal VaM logs,
or a real backup manifest. Build a synthetic reproducer or offer to coordinate
privately if the issue cannot otherwise be demonstrated.

You should receive an acknowledgment within seven days. Confirmed issues will
be assessed, fixed, and disclosed in proportion to their impact.

## Safety incidents

Unexpected replacement, archival, restore, or data-integrity behavior is
security-sensitive even when it is not an exploit. Stop using the affected
build, keep VaM closed, preserve the backup directory and reports, and report
the issue privately before attempting further cleanup.
