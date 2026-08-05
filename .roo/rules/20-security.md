<!-- SPDX-License-Identifier: MIT -->

# VaMender Security

- Treat terminal execution, file mutation, network access, and release operations as separate approvals.
- Prefer read-only inspection commands. Ask before any command that writes outside generated build output or changes Git history.
- Never reveal API keys, tokens, environment-variable values, private paths, or raw user package content in chat, logs, screenshots, or documentation.
- Do not use download-and-execute patterns, arbitrary shell pipelines, force pushes, hard resets, destructive cleanup, process termination, or publication commands.
- Preserve file ownership, path containment, checksum verification, independent backup, and no-network/content-rights boundaries.
- When handling screenshots, redact personal paths and delete temporary captures after producing the approved documentation asset.
