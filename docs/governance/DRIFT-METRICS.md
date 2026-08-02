<!-- SPDX-License-Identifier: MIT -->

# Drift Metrics

Track and fail or explicitly escalate:

- accepted requirements without a test mapping;
- tests referencing missing requirements;
- Markdown and `.specsmith` machine-state drift;
- user-facing names other than VaMender where a technical identifier is not
  intended;
- stable/GA language while REQ-001 defines the channel as beta;
- code paths that mutate without verified backup and restore evidence;
- release assets built outside GitHub Actions;
- version drift among tag, Cargo, installer, changelog, and filenames;
- required manual beta checks recorded as passed without environment evidence.
