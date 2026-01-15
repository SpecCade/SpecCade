# FG Audio V1 Library Expansion (RUNPACK)

This runpack is a prompt bundle for **Claude Code** to implement the audio-related “missing types” called out in `speccade/docs/FUTURE_GENERATORS.md`:

- All missing **Synthesis** types (Priority 1–3)
- All missing **Effect** types (Priority 1–3)
- All missing **LFO targets**
- All missing **Filter** types

## How to use (Claude Code)

1. Start Claude Code in the `speccade/` repo root.
2. Ensure the subagents in `speccade/.claude/agents/` are available.
3. Paste `speccade/.claude/runpacks/fg-audio-v1-library-expansion/ORCHESTRATOR.md` into the main chat as the “orchestrator” prompt.
4. The orchestrator should execute `FEATURE_INDEX.md` top-to-bottom, delegating each feature to subagents and marking progress.

## Resume / incremental runs

This runpack is designed to be resumable: check off completed items in `FEATURE_INDEX.md` and re-run the orchestrator prompt later.

## Claude Code command permissions

If Claude Code prompts for permission to run build/test commands (e.g., `cargo test`, `cargo clippy`), either approve them interactively or extend `speccade/.claude/settings.local.json` with the relevant `Bash(...)` allow-rules.

## What’s included

- `ORCHESTRATOR.md` — the single orchestrator prompt (dispatch + looping structure)
- `TEMPLATE_IMPLEMENT_FEATURE.md` — a reusable worker prompt template
- `FEATURE_INDEX.md` — checklist + canonical ordering
- `features/` — one prompt per missing type/target
