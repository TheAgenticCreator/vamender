# VaMender Release Readiness

Review release readiness without creating or publishing a release.

1. Inspect the worktree, current branch, changelog, release workflow, requirements, tests, and ledger.
2. Run `specsmith sync --check`, `specsmith req gaps`, `specsmith req orphans`, and `specsmith audit`.
3. Check the Windows quality, packaging, identity, security, documentation, and beta-acceptance evidence.
4. Separate passed evidence from pending CI or manual VaM acceptance.
5. Report blockers and exact next actions. Never claim release readiness when a required gate is unrun.

