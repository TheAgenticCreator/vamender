<!-- SPDX-License-Identifier: MIT -->

# Verification

Minimum local governance verification:

1. `specsmith sync --project-dir . --check`
2. `specsmith req gaps --project-dir .`
3. `specsmith req orphans --project-dir .`
4. `specsmith audit --project-dir .`

Minimum implementation verification is the mapped subset of
`docs/TESTS.md`. Merge candidates require the complete GitHub Actions CI and
CodeQL gates. Release candidates additionally require GitHub Actions packaging
and manual TEST-018 through TEST-022 in the supported environment.

Verification reports must distinguish passed, failed, and not-run checks.
Manual VaM tests cannot be claimed from this repository environment unless a
real supported VaM installation and disposable library were exercised.
