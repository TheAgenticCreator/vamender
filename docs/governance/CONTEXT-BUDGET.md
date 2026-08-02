<!-- SPDX-License-Identifier: MIT -->

# Context Budget

For each change, load `AGENTS.md`, the directly affected requirements and test
records, the relevant architecture component, and recent ledger entries first.
Load large implementation files by symbol or targeted section when possible.
Run `specsmith checkpoint` before a long task or handoff. Never trim backup,
restore, path-containment, sandbox, release-channel, or manual-gate constraints
from the active context.
