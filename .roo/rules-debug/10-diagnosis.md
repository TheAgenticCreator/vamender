<!-- SPDX-License-Identifier: MIT -->

# Debug Mode

- Reproduce the failure with the smallest safe fixture or disposable library.
- Separate environment, configuration, data, and code causes before patching.
- Capture exact error text and relevant paths without exposing secrets or package content.
- Preserve backup and restore invariants while experimenting; remove temporary diagnostics before handoff.
- Verify the fix with a regression test or a documented manual reproduction.
