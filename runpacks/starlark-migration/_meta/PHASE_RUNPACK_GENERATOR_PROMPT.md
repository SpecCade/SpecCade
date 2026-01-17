# Prompt: Generate a Phase Runpack (Recursive)

Give this file to an agent to generate a **single phase runpack** under:

`runpacks/starlark-migration/phases/phase-<ID>-<slug>/`

This is designed for **context efficiency**: the agent should read only a small, bounded set of files and produce a fully self-contained phase runpack that can later be executed (possibly by another agent) without relying on chat history.

Assume the primary orchestrator is **Claude Code** (able to spawn subtasks/subagents).

This runpack is intended for "coordinator-only" orchestration:
- The main orchestrator coordinates and writes artifacts under the phase folder.
- Code edits may happen only inside the `20_implement` and `40_quality` subtasks.
- Commands/tests may happen only inside the `30_validate` subtask.

If subtasks/subagents are unavailable, the orchestrator should STOP and ask the user to rerun in an environment that supports tasks (do not silently fall back to the orchestrator doing the work).

---

## Inputs

You MUST read:

- `runpacks/starlark-migration/PHASES.yaml`
- `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md`

Then you MUST be given (by the orchestrator/user):

- `PHASE_ID` (integer)

You MUST derive the phase metadata from `runpacks/starlark-migration/PHASES.yaml` for the given `PHASE_ID`:
- `slug` (directory name component; must match exactly)
- `title`, `goal`
- `scope_globs`, `acceptance_criteria`, `validation_commands`, `notes` (if present)

---

## Output: required files

Create the directory:

`runpacks/starlark-migration/phases/phase-<PHASE_ID>-<slug>/`

The `<slug>` MUST exactly match the `slug` for `PHASE_ID` in `PHASES.yaml` (do not invent/rename it).

You MUST NOT edit repo code or run commands while generating the phase runpack.
Only create/update files under `runpacks/starlark-migration/phases/phase-<PHASE_ID>-<slug>/**`.

And write these files (exact names):

1. `runpacks/starlark-migration/phases/phase-<...>/ORCHESTRATOR.md`
2. `runpacks/starlark-migration/phases/phase-<...>/STATUS.md`
3. `runpacks/starlark-migration/phases/phase-<...>/SCOPING.md`
4. `runpacks/starlark-migration/phases/phase-<...>/ARTIFACTS.md`
5. `runpacks/starlark-migration/phases/phase-<...>/prompts/00_research.md`
6. `runpacks/starlark-migration/phases/phase-<...>/prompts/10_plan.md`
7. `runpacks/starlark-migration/phases/phase-<...>/prompts/20_implement.md`
8. `runpacks/starlark-migration/phases/phase-<...>/prompts/30_validate.md`
9. `runpacks/starlark-migration/phases/phase-<...>/prompts/40_quality.md`

No other files are required for generation (keep it lean), but you may add:

- `prompts/99_handoff.md` if the phase is large

---

## Rules (robustness > speed)

1) Artifact-driven:
- Every prompt MUST instruct the agent to write its outputs to specific files in this phase folder.
- The phase can be resumed from disk with minimal re-reading.

2) Scope safety:
- `SCOPING.md` MUST list allowed globs and must-not-touch guidance for this phase.
- Prompts MUST instruct implementers to avoid touching out-of-scope files unless required, and to record justification.

3) No "silent" decisions:
- Any change to earlier architectural decisions MUST be recorded in `ARTIFACTS.md` (decision log section).

4) Validation is mandatory:
- `30_validate.md` MUST include the phase's `validation_commands` from `PHASES.yaml` and require recording outputs.

5) Token efficiency:
- Prompts MUST include a "Files to open first" list and forbid wide repo scans unless blocked.

6) Claude Code subagent friendliness:
- `ORCHESTRATOR.md` MUST include:
  - a short "Recommended dispatch plan" (which prompts can run in parallel vs must be exclusive)
  - a strict "Subagent protocol" (what each subagent may do, and what artifacts it must write)
  - a "Coordinator-only rule" (the main orchestrator must not edit code or run commands)

---

## Content requirements (what each file must contain)

### ORCHESTRATOR.md
- A deterministic checklist of steps:
  1) run research prompt
  2) run planning prompt
  3) run implementation prompt
  4) run validation prompt
  5) run quality prompt
  6) finalize phase summary + mark complete
- A clear instruction: each step above MUST run as a subtask/subagent (do not perform the role in the main orchestrator thread).
- An explicit "Coordinator-only rule" section:
  - main orchestrator may only write under this phase folder
  - main orchestrator must not apply patches outside this phase folder
  - main orchestrator must not run build/test commands
- Explicit "stop conditions" (when to ask for help vs proceed with assumptions).
- A "Recommended dispatch plan" section:
  - Research/plan may be parallel read-only (if tooling supports)
  - Implement/validate/quality must be exclusive (avoid edit conflicts)

### STATUS.md
- A checkbox list for each stage and key acceptance criteria.
- A "current blockers" section.

### SCOPING.md
- Allowed file globs (from `PHASES.yaml`).
- Must-not-touch suggestions (e.g. don't refactor unrelated backends).
- Safety notes: determinism, hashing, schema stability.

### ARTIFACTS.md
- Paths to the phase-produced artifacts (the files the prompts will write).
- A "Decision log" section with date + rationale entries.

### prompts/00_research.md
- Role: strictly read/understand; no code edits.
- Must include an explicit permission boundary: do not apply patches; do not run commands.
- Must output:
  - `research.md` (notes)
  - `questions.md` (only if blocking)
  - `risks.md`

### prompts/10_plan.md
- Role: produce an implementable plan, file list, APIs.
- Must include an explicit permission boundary: do not apply patches; do not run commands.
- Must output:
  - `plan.md`
  - `interfaces.md` (new structs/commands)
  - `test_plan.md`

### prompts/20_implement.md
- Role: implement only the planned work.
- Must include an explicit permission boundary: code edits allowed; do not run build/test commands (validation happens in `30_validate`).
- Must output:
  - `implementation_log.md`
  - `diff_summary.md`

### prompts/30_validate.md
- Role: run validation commands, capture outputs, triage failures.
- Must include an explicit permission boundary: commands allowed; do not apply patches.
- Must output:
  - `validation.md`
  - `failures.md` (if needed)

### prompts/40_quality.md
- Role: refactor for maintainability, reduce complexity, improve prompt efficiency.
- Must include an explicit permission boundary: code edits allowed; do not run build/test commands.
- Must output:
  - `quality.md`
  - `followups.md` (optional)

---

## Final instruction

Generate these files for `PHASE_ID`, derived from `PHASES.yaml`, with minimal boilerplate and maximal clarity.
