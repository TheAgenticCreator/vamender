# VaMender Governance

- Treat `AGENTS.md` as mandatory project policy. Read it before repository work.
- The canonical product name is VaMender. Keep all release and user-facing language beta-accurate.
- Before changing behavior, read `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, and the relevant governance documents.
- Map product, governance, release, or security changes to accepted `REQ-NNN` records and preserve evidence.
- Treat `docs/REQUIREMENTS.md` and `docs/TESTS.md` as the documented sources of truth; run `specsmith sync` after changing either.
- Never weaken backup-before-mutation, checksum, restore, path-containment, plugin-sandbox, content-rights, or independent-backup protections.
- Never run destructive VaMender operations against a user's only AddonPackages copy. Use a disposable library for acceptance work.
- GitHub Actions is the release authority. Do not assemble or claim release artifacts from the workstation.
- Do not commit, push, create tags, or create branches unless the user explicitly requests that exact action.

