<!-- SPDX-License-Identifier: MIT -->

# Contributing to VaMender

Thank you for helping make VAR maintenance safer.

## Before contributing

Read [SECURITY.md](SECURITY.md), [DISCLAIMER.md](DISCLAIMER.md), and the safety
model in [README.md](README.md). Never test a development build against the
only copy of an `AddonPackages` library.

Do not upload or commit:

- paid, private, or otherwise redistributable VAR packages;
- creator assets without explicit permission;
- VaM logs containing personal paths, usernames, tokens, or private URLs;
- live backup manifests or real user library inventories.

Use small synthetic ZIP/VAR fixtures that you created and are permitted to
share.

## Development workflow

1. Open an issue for behavior-changing work so the safety implications can be
   discussed first.
2. Fork the repository and create a focused branch from `main`.
3. Add or update tests for the change.
4. Run the same formatting, lint, automated test, and Windows build checks
   required by CI.
5. Open a pull request from your branch into `main`.
6. Describe the safety behavior, backup behavior, and test evidence in the
   pull request.

Pull requests must be current with `main`. Required checks and an approving
review must pass before merge. Force pushes and deletion of `main` are
disabled.

## Public commit identity and contributor credit

VaMender's canonical public history uses the publisher identity
`TheAgenticCreator <312204356+TheAgenticCreatorDev@users.noreply.github.com>`.
Maintainers must run `./tools/configure-maintainer-identity.ps1` in their
checkout before creating commits or annotated tags, and must authenticate as
GitHub user `TheAgenticCreatorDev` or an approved organization GitHub App. A
personal GitHub credential remains visible as the authenticated actor even if
Git author fields are changed.

External contributors should keep their normal identity in their fork and pull
request. When a contribution is incorporated into canonical history, its
credit should be retained with one or more `Co-authored-by` trailers while the
project-authored commit keeps the VaMender pseudonym.

Applied operations must remain reversible. New write paths must back up and
verify the original before replacing or moving it, record the operation in
the restore manifest, fail closed when evidence is ambiguous, and preserve
strict script/plugin version requirements.

By contributing, you agree that your contribution is licensed under the MIT
License and that you will follow the [Code of Conduct](CODE_OF_CONDUCT.md).

Project ownership and maintainer responsibilities are documented in
[MAINTAINERS.md](MAINTAINERS.md).
