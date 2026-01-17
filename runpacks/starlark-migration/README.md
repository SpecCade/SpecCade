# Starlark Migration Runpack (Orchestrator-Agnostic)

This folder contains **recursive runpacks** to migrate SpecCade from **JSON-only specs** to **Starlark authoring compiled into canonical JSON IR**, without breaking existing users.

## What "recursive runpacks" means

- You feed **one prompt file** to an orchestrator agent.
- That agent **generates the Phase 1 runpack**, executes it, writes durable artifacts, then generates **Phase 2**, etc.
- Every phase produces **on-disk artifacts** so progress survives context loss / compaction.

## Start here

- Primary prompt to give to an orchestrator agent:
  - `runpacks/starlark-migration/_meta/MAIN_ORCHESTRATOR_PROMPT.md`
- Phase definitions (scope, acceptance, validation commands):
  - `runpacks/starlark-migration/PHASES.yaml`
- Architecture / contract reference (SSOT for the migration):
  - `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md`

## Recommended dispatch (optional)

This runpack is designed to work with **one** tool-using agent (Claude Code works well),
but you can split work into role-focused subtasks per phase. Each generated phase runpack includes:

- `prompts/00_research.md` (read-only)
- `prompts/10_plan.md` (read-only)
- `prompts/20_implement.md` (code edits)
- `prompts/30_validate.md` (runs commands/tests)
- `prompts/40_quality.md` (refactor + polish)

Intended workflow: the main orchestrator coordinates and dispatches these prompts as subtasks; it should not write code itself.

## Output location

Generated per-phase runpacks and artifacts go under:

`runpacks/starlark-migration/phases/phase-<N>-<slug>/`
