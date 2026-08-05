<!-- SPDX-License-Identifier: MIT -->

# VaMender Workflow

- Start by inspecting `git status`, the relevant requirements, architecture, and current Specsmith state.
- For a resumed project, begin in Architect mode for a read-only state and release-readiness assessment, then switch to Code only after the plan is clear.
- Use the project router aliases rather than raw provider model IDs: `local-fast`, `local-code`, `local-vision`, `cloud-qwen-plus`, and `cloud-qwen-max`.
- Prefer `local-fast` for questions and lightweight triage, `local-code` for routine implementation, `local-vision` for screenshots or image inspection, `cloud-qwen-plus` for architecture and documentation, and `cloud-qwen-max` for difficult reasoning, orchestration, or final review.
- Keep cloud use minimal and deliberate. Do not send secrets, credentials, personal filesystem paths, private package content, or unnecessary full-repository blobs to cloud models.
- Make the smallest root-cause change. Use `apply_patch` for edits and avoid unrelated cleanup.
- After editing, run the narrowest relevant checks first, then the proportional Rust, plugin, documentation, and Specsmith gates.
- Report unrun required checks and existing unrelated failures explicitly; never convert an incomplete beta gate into a release claim.
