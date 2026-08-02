<!-- SPDX-License-Identifier: MIT -->

# Governance Rules

1. `docs/REQUIREMENTS.md` and `docs/TESTS.md` are the Markdown sources of truth;
   `.specsmith` JSON files are derived by `specsmith sync`.
2. Governance changes require a scoped proposal and human approval. An explicit
   user instruction authorizing that exact change is approval.
3. Every behavioral, release, or governance change maps to at least one
   accepted requirement and corresponding evidence.
4. `LEDGER.md` is append-only. Corrections are new entries, never rewrites.
5. Production-grade safety invariants remain mandatory during beta.
6. Environment-dependent beta acceptance cannot be replaced by unit tests or a
   green CI badge; an unrun required check remains an explicit blocker.
7. Release artifacts come from GitHub Actions running on the tagged commit.
