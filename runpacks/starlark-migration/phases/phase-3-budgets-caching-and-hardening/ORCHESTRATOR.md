# Phase 3 Orchestrator: Budgets, Caching, and Hardening

## Phase Goal

Unify hard budgets across Starlark and JSON IR inputs, improve caching keys and reports, and add robustness tests (idempotence, fuzz-ish validation).

## Coordinator-Only Rule

The main orchestrator thread:
- MAY only write/update files under `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/`
- MUST NOT apply patches to any files outside this phase folder
- MUST NOT run build, test, or validation commands
- MUST delegate all implementation/validation work to subtasks/subagents

If subtasks/subagents are unavailable, STOP and ask the user to rerun in an environment that supports tasks.

## Checklist (execute in order)

1. [ ] **Research** - Spawn subtask with `prompts/00_research.md`
   - Wait for `research.md`, `risks.md`, optionally `questions.md`
   - If `questions.md` has blocking items, pause and escalate to user

2. [ ] **Plan** - Spawn subtask with `prompts/10_plan.md`
   - Wait for `plan.md`, `interfaces.md`, `test_plan.md`
   - Review plan against acceptance criteria before proceeding

3. [ ] **Implement** - Spawn subtask with `prompts/20_implement.md`
   - Wait for `implementation_log.md`, `diff_summary.md`
   - This step edits code; must run exclusively (no parallel tasks)

4. [ ] **Validate** - Spawn subtask with `prompts/30_validate.md`
   - Wait for `validation.md`, optionally `failures.md`
   - If failures exist, iterate with implementer or escalate

5. [ ] **Quality** - Spawn subtask with `prompts/40_quality.md`
   - Wait for `quality.md`, optionally `followups.md`
   - This step may edit code; must run exclusively

6. [ ] **Finalize** - Update `STATUS.md` to mark phase complete
   - Summarize outcomes in `ARTIFACTS.md` decision log
   - Record any deferred work in `followups.md`

## Recommended Dispatch Plan

| Prompt | Parallelizable | Notes |
|--------|----------------|-------|
| `00_research` | Yes (read-only) | Can run with other read-only tasks |
| `10_plan` | Yes (read-only) | Can run after research completes |
| `20_implement` | **Exclusive** | Edits code; no parallel tasks |
| `30_validate` | **Exclusive** | Runs commands; wait for implement |
| `40_quality` | **Exclusive** | Edits code; wait for validate pass |

## Subagent Protocol

Each subagent:
1. MUST read `SCOPING.md` before touching any files
2. MUST write outputs to `runpacks/starlark-migration/phases/phase-3-budgets-caching-and-hardening/`
3. MUST NOT edit files outside scope globs (see `SCOPING.md`)
4. MUST record any architectural deviations in `ARTIFACTS.md` decision log

## Stop Conditions

Pause and ask for user input if:
- `questions.md` contains blocking unknowns
- Validation fails repeatedly (>2 cycles)
- Implementation requires touching files outside scope globs
- Budget/caching design conflicts with existing `speccade_spec::hash` semantics
