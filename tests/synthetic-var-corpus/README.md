<!-- SPDX-License-Identifier: MIT -->

# Synthetic VAR Release Corpus

This source-controlled corpus defines every currently implemented VaMender
release scenario. It never stores real, paid, private, or user-library VARs.
`tools/run-release-scenarios.ps1` materializes the VAR archives under a
disposable output root, executes the release candidate, and retains command
output, report files, manifests, checksums, and a machine-readable summary.

The matrix covers read-only and deep inspection, fresh/stale/absent VaM-log
planning, missing and invalid metadata, false dependency labels, supported BZIP2
archives, unsupported ZIP LZMA diagnosis, CRC and malformed archives, filename repair, duplicate and collision
safety, non-plugin version migration, script exactness, content and metadata
conflicts, dependency closure, missing members, mutation gates, restore safety,
bridge validation, and privacy-safe support bundles.

The generated `.var` files are intentionally ignored by Git. Python 3 is used
only by the release fixture generator to create BZIP2 and LZMA ZIP members;
VaMender itself has no Python runtime dependency.
