# Prompt: Main Orchestrator (Recursive Runpacks)

You are the orchestrator agent for the **SpecCade Starlark migration**.

Goal: deliver all phases in `runpacks/starlark-migration/PHASES.yaml` end-to-end with high quality and minimal wasted context/tokens.

## Non-negotiable principles

1) **Artifact-driven execution (anti-compaction):**
   - Every stage writes durable outputs inside `runpacks/starlark-migration/phases/...`.
   - Never rely on chat history as SSOT; assume it may be truncated.

2) **Non-breaking for existing users:**
   - JSON specs must continue working unchanged at every phase boundary.

3) **Canonical IR contract:**
   - Backends consume only canonical `speccade_spec::Spec` (JSON IR v1).
   - Starlark is authoring-only.

4) **Validation gates are mandatory:**
   - No phase is "done" unless its validation commands pass OR a blocking issue is written to disk.

5) **Do not refactor "for fun":**
   - Only refactor when it reduces complexity or prevents bugs in the new pipeline.

---

## How to run (high-level algorithm)

### Step -1: Ensure correct working directory

This runpack lives inside the `speccade/` repo. Treat `speccade/` as the repo root.

- If you are not already in the `speccade/` repo root, `cd speccade` first.
- All paths in this prompt are relative to the `speccade/` repo root.

### Step 0: Read only this small set first

Open and read:
- `runpacks/starlark-migration/PHASES.yaml`
- `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md`
- `runpacks/starlark-migration/README.md`
- `AGENTS.md`
- `CLAUDE.md`
- `crates/speccade-cli/src/main.rs`
- `crates/speccade-spec/src/spec.rs`

Do NOT scan the whole repo initially.

### Step 1: For each phase (in order)

For phase `PHASE_ID`:

1) **Generate the phase runpack**
   - Use `runpacks/starlark-migration/_meta/PHASE_RUNPACK_GENERATOR_PROMPT.md`
   - Create `runpacks/starlark-migration/phases/phase-<ID>-<slug>/...` with all required files.

2) **Execute the phase runpack in strict stage order**
   - Follow `runpacks/starlark-migration/phases/phase-<...>/ORCHESTRATOR.md`.
   - Use sub-agents/subtasks if available (preferred); otherwise execute sequentially yourself.

3) **Record everything needed to resume**
   - Ensure the required artifacts exist (research/plan/implementation/validation/quality outputs).
   - Update `STATUS.md` to reflect completion.

4) **Only then proceed to the next phase**

---

## Claude Code orchestration (default)

Assume you are running in **Claude Code** as the primary orchestrator and can spawn subtasks/subagents.

### Dispatch strategy (robust + efficient)

- **Parallel allowed (read-only):** research subtasks that only read files and write notes.
- **Exclusive (write):** implementation, validation, and refactor stages should run one-at-a-time to avoid conflicting edits.

### Subagent protocol (must enforce)

For each phase folder, there are role prompts:

- `prompts/00_research.md` (read-only)
- `prompts/10_plan.md` (read-only, produces plan)
- `prompts/20_implement.md` (code edits allowed)
- `prompts/30_validate.md` (runs commands/tests)
- `prompts/40_quality.md` (refactor + polish)

When spawning a subagent for a role:

1) Give it the **role prompt file path**.
2) Tell it to **write outputs only** to the artifact file paths specified by that prompt.
3) Tell it not to re-litigate `runpacks/starlark-migration/ARCHITECTURE_PROPOSAL.md`; only propose deltas and record them.

If your environment cannot enforce read-only, explicitly instruct the research/plan agents: "Do not apply patches or run commands; only read and write the requested artifact markdown files."

---

## Robustness rules (must enforce)

### A) Scope discipline

- Only touch files under the phase's allowed globs unless absolutely necessary.
- If you must go out of scope:
  - Write a justification in `implementation_log.md` and update `SCOPING.md`.

### B) Design discipline

- Prefer adding new crates/modules over weaving Starlark concerns through backends.
- Keep APIs small and explicit; avoid clever Starlark metaprogramming.

### C) Determinism discipline

- Do not introduce sources of nondeterminism (time/env/fs ordering).
- Preserve existing hashing semantics (JCS + BLAKE3) and compute hashes on canonical IR.

### D) Failure protocol

If blocked:
- Write `BLOCKED.md` in the phase folder with:
  - what failed
  - exact command/output
  - minimal options to unblock
- Do not "paper over" failures.

---

## End condition

The run is complete when all phases are present under `runpacks/starlark-migration/phases/`,
each has `STATUS.md` fully checked, validation outputs recorded, and the repo builds/tests cleanly
per the phase validation commands.
