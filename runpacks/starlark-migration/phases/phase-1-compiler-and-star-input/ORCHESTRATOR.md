# Phase 1 Orchestrator: Canonical Compiler Pipeline + .star Input

## Phase metadata

- **ID**: 1
- **Slug**: compiler-and-star-input
- **Title**: Phase 1: Canonical compiler pipeline + .star input (non-breaking)
- **Goal**: Introduce a compiler pipeline that accepts Starlark specs and produces the existing canonical JSON IR (Spec v1), while keeping JSON specs fully supported.

---

## Coordinator-only rule

The main orchestrator (this thread) MUST:
- Only write files under `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/`
- NOT apply patches to repo code (delegate to subagents)
- NOT run build/test commands (delegate to subagents)
- Dispatch each step below as a subtask/subagent

If subtasks/subagents are unavailable, STOP and ask the user to rerun in an environment that supports tasks.

---

## Execution checklist

Run each step as a **subtask/subagent**. Do not perform the work inline.

### Step 1: Research (read-only)
- [ ] Dispatch `prompts/00_research.md`
- [ ] Verify outputs exist: `research.md`, `risks.md`, optionally `questions.md`
- [ ] If `questions.md` contains blocking questions, STOP and escalate to user

### Step 2: Plan (read-only)
- [ ] Dispatch `prompts/10_plan.md`
- [ ] Verify outputs exist: `plan.md`, `interfaces.md`, `test_plan.md`
- [ ] Review plan for scope violations; if found, STOP and escalate

### Step 3: Implement (code edits allowed)
- [ ] Dispatch `prompts/20_implement.md`
- [ ] Verify outputs exist: `implementation_log.md`, `diff_summary.md`

### Step 4: Validate (commands allowed)
- [ ] Dispatch `prompts/30_validate.md`
- [ ] Verify outputs exist: `validation.md`, optionally `failures.md`
- [ ] If `failures.md` indicates critical failures, return to Step 3 or escalate

### Step 5: Quality (code edits allowed)
- [ ] Dispatch `prompts/40_quality.md`
- [ ] Verify outputs exist: `quality.md`, optionally `followups.md`

### Step 6: Finalize
- [ ] Update `STATUS.md` to mark all criteria complete
- [ ] Record any decision changes in `ARTIFACTS.md`
- [ ] Summarize phase completion for user

---

## Recommended dispatch plan

| Prompt | Mode | Dependencies | Notes |
|--------|------|--------------|-------|
| `00_research.md` | read-only | none | Can run in parallel with nothing |
| `10_plan.md` | read-only | research complete | Needs research artifacts |
| `20_implement.md` | code edits | plan complete | Exclusive (no parallel edits) |
| `30_validate.md` | commands | implement complete | Exclusive |
| `40_quality.md` | code edits | validate complete | Exclusive |

Parallel dispatch is only safe for research + plan if tooling supports read-only isolation.

---

## Subagent protocol

Each subagent MUST:
1. Read the prompt file it is dispatched with
2. Read `SCOPING.md` to understand boundaries
3. Perform only actions permitted by its prompt
4. Write all required output artifacts to this phase folder
5. NOT assume chat history; rely only on disk artifacts

Each subagent MUST NOT:
- Edit files outside scope globs (see `SCOPING.md`)
- Run commands unless explicitly permitted by its prompt
- Make architectural decisions without recording them

---

## Stop conditions

STOP and escalate to user if:
- `questions.md` contains blocking questions after research
- Plan violates scope or architectural constraints
- Validation fails with > 3 test failures after one retry cycle
- Any subagent reports inability to proceed

PROCEED with assumptions if:
- Minor ambiguities that don't affect correctness
- Optional features can be deferred to Phase 2
- Non-blocking warnings in validation
