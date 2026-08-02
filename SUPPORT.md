<!-- SPDX-License-Identifier: MIT -->

# Support

For usage questions and reproducible bugs, open a GitHub issue using the
appropriate template.

Before posting:

1. Back up the complete `AddonPackages` directory somewhere independent.
2. VaM may remain open, but stop scene playback and avoid loading content from
   VARs that may be modified.
3. Run a fresh VaM package rescan, then create a local bundle with
   `vamender support-report <AddonPackages> --vam-log <output_log.txt>`.
4. Read `README_FIRST.txt` and review every generated file before attaching the
   ZIP. Package identifiers can disclose paid or private content.
5. Add `--include-var-list` only when the complete installed-package inventory
   is relevant and you consent to sharing it.
6. Remove usernames, personal paths, tokens, and private URLs from anything you
   paste separately. Do not attach full VaM logs, paid/private VARs, modified
   third-party VARs, or real backup manifests.

`support-report` extracts package-related lines instead of copying the full VaM
log. It does not include VAR payloads, absolute paths, credentials, or backup
manifests. It never uploads anything. `--open-github --confirm-reviewed` only
opens the issue form after you confirm that you reviewed the local files; you
still choose whether to attach and submit them.

Use GitHub's private vulnerability reporting for security or unexpected
data-integrity problems. VaMender is community software provided without
warranty. Beta support is best-effort; the evidence required for v1.0.0 is
tracked in [docs/ROADMAP.md](docs/ROADMAP.md).
