<!-- SPDX-License-Identifier: MIT -->

# Maintainers

VaMender's maintainers safeguard the project's backup-first design, review
changes, coordinate releases, and help contributors make improvements without
weakening data-integrity or licensing protections.

## Current maintainers

| Maintainer | GitHub | Responsibilities |
| --- | --- | --- |
| TheAgenticCreator | [@TheAgenticCreatorDev](https://github.com/TheAgenticCreatorDev) and [TheAgenticCreator organization](https://github.com/TheAgenticCreator) | Project direction, code review, security response, and releases |

## Maintainer pseudonym

Public project-authored commits and annotated tags use
`TheAgenticCreator <312204356+TheAgenticCreatorDev@users.noreply.github.com>`.
Maintainers configure that identity locally with
`tools/configure-maintainer-identity.ps1` and authenticate GitHub operations as
`TheAgenticCreatorDev` or an approved organization GitHub App. A personal
account must not be used to push, merge, tag, or create releases because
GitHub event metadata records the authenticated actor independently of Git
author and committer fields.

## Maintainer responsibilities

- Require a tested, restorable backup path for every write-capable operation.
- Preserve strict handling of scripts/plugins and fail closed when package
  compatibility cannot be established.
- Review dependency-resolution changes against synthetic fixtures and VaM's
  own rescan evidence.
- Keep protected-branch, CI, CodeQL, release, SemVer, changelog, and SPDX
  policies operational.
- Triage security and data-integrity reports privately and responsibly.
- Welcome focused contributions from users and other VAR-manager developers.

Repository governance is enforced through protected main branch review and
status-check rules. Security-sensitive reports should follow
[SECURITY.md](SECURITY.md), not a public issue.
