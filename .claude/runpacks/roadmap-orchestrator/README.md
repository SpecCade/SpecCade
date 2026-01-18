# Roadmap Orchestrator (RUNPACK)

This runpack is a prompt bundle for **Claude Code** to execute SpecCade's development roadmap in `docs/ROADMAP.md` using **multiple parallel subagents** (builder/refactor/verifier) per task.

It is designed to:
- Always pick exactly **one** next roadmap item.
- Provide a tight "task brief" and definition of done (DoD).
- Loop until the DoD is met (no "30% done" stops).
- Commit/push per completed task to stay resumable.

## How to use (Claude Code)

1. Start Claude Code in the `speccade/` repo root (the folder containing `.claude/`).
2. Ensure the subagents in `.claude/agents/` are available.
3. Paste `.claude/runpacks/roadmap-orchestrator/ORCHESTRATOR.md` into the main chat as the orchestrator prompt.
4. The orchestrator will iterate `docs/ROADMAP.md` until everything is checked off (or it hits a decision that needs user confirmation).

## Resumable runs

This runpack is resumable by design:
- `docs/ROADMAP.md` checkboxes are the source of truth.
- Re-run the orchestrator prompt at any time; it will pick the next unchecked item.

## Permissions

If Claude Code prompts for permission to run build/test commands (e.g. `cargo test`, `cargo clippy`) or git commands, either approve them interactively or extend `.claude/settings.local.json` with the relevant allow-rules.

