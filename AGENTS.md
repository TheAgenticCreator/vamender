<!-- SPDX-License-Identifier: MIT -->

# VaMender Agent Governance

## Local identity and release posture

- The canonical product and project name is **VaMender**.
- `vamender` is the Rust crate, executable, and CLI command identifier.
- VaM package identifiers remain `AgenticCreator.VaMender`.
- The public maintainer and publisher identity is **TheAgenticCreator** with
  commit identity
  `TheAgenticCreator <312204356+TheAgenticCreatorDev@users.noreply.github.com>`.
  Never publish a maintainer's legal name, personal account name, or personal
  email in repository history.
- Maintainer pushes must authenticate as GitHub user `TheAgenticCreatorDev` or
  an approved organization GitHub App. Commit metadata alone does not hide
  GitHub's authenticated push actor.
- Treat all current releases as **beta** until an approved requirement changes
  the channel. The implementation target is production-grade safety and
  evidence even while the channel remains beta.
- Windows x64 with Virt-a-Mate 1.22.0.12 is the supported product runtime.

## Required workflow

1. Read this file fully before starting repository work.
2. Read the applicable requirements in `docs/REQUIREMENTS.md` and architecture
   in `docs/ARCHITECTURE.md` before changing behavior.
3. Propose governance changes and obtain human approval. An explicit user
   request to make the scoped governance change counts as approval.
4. Map every change to one or more `REQ-NNN` records.
5. Update or add evidence mappings in `docs/TESTS.md`.
6. Run `specsmith sync` after requirements or test-spec changes.
7. Verify proportionally to risk and run `specsmith audit` before completion.
8. Append an entry to `LEDGER.md`; never rewrite prior ledger entries.
9. Before maintainer commits or tags, run
   `tools/configure-maintainer-identity.ps1` and keep its pre-push guard active.

## Safety invariants

- Never weaken backup-before-mutation, checksum, restore, path-containment,
  plugin-sandbox, content-rights, or independent-backup protections merely to
  make a test or release pass.
- Never run destructive VaMender operations against a user's only
  AddonPackages copy. Manual beta acceptance uses a disposable library copy.
- Do not claim a beta is generally available or production-proven.
- GitHub Actions is the release authority: release artifacts must be built,
  packaged, checksummed, and uploaded from the tagged commit by the release
  workflow, not assembled ad hoc on a maintainer workstation.
