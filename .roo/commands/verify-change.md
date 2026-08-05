# Verify VaMender Change

Verify a focused change without committing it.

1. Identify the affected REQ-NNN records and the narrowest relevant tests.
2. Run the appropriate Specsmith preflight and focused test or build checks.
3. Run formatting and lint checks proportionally to the change.
4. Run `specsmith audit` and inspect the final diff and worktree.
5. Report commands, results, unrelated failures, and any remaining manual evidence.

