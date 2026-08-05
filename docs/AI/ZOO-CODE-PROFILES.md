<!-- SPDX-License-Identifier: MIT -->

# VaMender Zoo Code Profiles

This project uses the local AI router at `http://127.0.0.1:4100`. The router owns cloud credentials and performs local model lifecycle switching, so Zoo Code should use the native LiteLLM provider and router aliases.

## Profiles

| Profile | Router model | Default use |
| --- | --- | --- |
| VaMender - Local Fast | `local-fast` | Ask mode and lightweight triage |
| VaMender - Local Code | `local-code` | Routine Rust, C#, PowerShell, and documentation changes |
| VaMender - Local Vision | `local-vision` | Screenshots, image inspection, and visual documentation work |
| VaMender - Cloud Plus | `cloud-qwen-plus` | Architecture, documentation, and cost-conscious cloud reasoning |
| VaMender - Cloud Max | `cloud-qwen-max` | Hard debugging, orchestration, and final review |
| Kith - Local | `kith` | Kith gateway (OpenAI-compatible, local GPU via Ollama) |

## Mode assignments

| Zoo mode | Assigned profile | Advanced settings |
| --- | --- | --- |
| Ask | Local Fast | Reasoning on, 16K output, temperature 0.2 |
| Architect | Cloud Plus | Reasoning on, 32K output, temperature 0.15, prompt caching on |
| Code | Local Code | Reasoning off, 32K output, temperature 0.1 |
| Debug | Local Code | Reasoning off, 32K output, temperature 0.1 |
| Orchestrator | Cloud Max | Reasoning on, 64K output, 16K thinking budget, prompt caching on |
| Review | Cloud Max | Read-only, reasoning on, 64K output, 16K thinking budget, prompt caching on |

All VaMender profiles use:

- Provider: `LiteLLM`
- Base URL: `http://127.0.0.1:4100`
- API key: `dummy-key`
- Prompt caching: enabled only for the two cloud profiles
- Model IDs: router aliases above, not raw Ollama or Qwen IDs

The Kith profile uses:

- Provider: `OpenAI` (OpenAI-compatible)
- Base URL: `http://127.0.0.1:8001/v1`
- API key: `KITH_API_KEY` from the kith-ai `.env` file
- Model ID: `kith`
- Prompt caching: disabled (local gateway)

Prompt caching is intentionally disabled for the local Ollama profiles because those models do not expose a compatible remote cache. It is enabled for Cloud Plus and Cloud Max because the router metadata marks those Qwen cloud endpoints as cache-capable.

The cloud profile resolves to `qwen3.8-max` through the router. Do not replace it with `qwen3.8-max-preview`.

## Import

Zoo Code stores provider profiles in VS Code Secret Storage, so configure those through the Zoo panel rather than editing workspace files:

1. Open the Zoo panel, choose the gear icon, and scroll to the bottom of Settings.
2. Use `Import Settings` to import `docs/AI/vamender-zoo-code-profiles.json`, or create the six profiles from the table under `Providers` if the installed build rejects imports.
3. Confirm the five VaMender profiles use provider `LiteLLM`, base URL `http://127.0.0.1:4100`, and API key `dummy-key`. Confirm the Kith profile uses provider `OpenAI`, base URL `http://127.0.0.1:8001/v1`, and the `KITH_API_KEY` from the kith-ai `.env` file.
4. Open the Prompts tab and assign each mode using the Mode assignments table. This is the supported profile-to-mode association flow.
5. Leave only the six profiles (five VaMender + Kith) and refresh model discovery with the routers running.

The project Review mode is defined in `.roomodes` and its rules are in `.roo/rules-review/`; reload the workspace after changing modes.

The import template maps Ask, Architect, Code, Debug, and Orchestrator to sensible defaults. A task can still be switched manually to Local Vision, Cloud Max, or Kith when the task needs images, deeper reasoning, or the Kith gateway's local GPU models.

## Resume workflow

Start a resumed project task in Architect mode with `VaMender - Cloud Plus` or `VaMender - Local Fast` if the task is small. After the state and plan are clear, switch to Code with `VaMender - Local Code`. Use Cloud Max only for difficult failures, cross-cutting design decisions, or the final review.
