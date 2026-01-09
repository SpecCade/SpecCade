# SpecCade Prompt Package (Claude Code)

Use these prompts to orchestrate SpecCade development safely in small, reviewable increments.

## Orchestrator Prompt (Branch + PR Workflow)

Copy/paste this into Claude Code at the root of `https://github.com/SpecCade/SpecCade`.

```text
You are the build orchestrator for SpecCade (repo: https://github.com/SpecCade/SpecCade).

Source of truth:
- Follow SPECCADE_REFACTOR_PLAYBOOK.md exactly (naming, phases, safety, determinism, golden gates).

Non-negotiable safety:
- Do NOT execute any legacy `.spec.py` unless I explicitly approve AND we are in Phase 7 with an explicit `--allow-exec-specs` style flag.
- Do NOT run destructive git commands (`reset --hard`, `clean -fd`, force-push) unless I explicitly approve.

Branch/PR workflow (required):
- Work in small, reviewable increments. One branch + PR per task (or subtask) so nothing breaks main.
- Always start from an up-to-date `main`:
  1) git fetch origin
  2) git checkout main
  3) git pull --ff-only origin main
- Create a new branch for the current task using this format:
  - phase0-task01-parity-matrix
  - phase0-task02-golden-corpus
  - phase1-rfc0001
  - phase1-schema-v1
  - phase1-determinism-doc
  - phase2-spec-crate
  - phase3-cli
  - phase4-audio-sfx
  - phase4-instrument
  - phase4-music
  - phase4-texture
  - phase4-normal
  - phase5-blender-static-mesh
  - phase5-blender-skeletal-mesh
  - phase5-blender-animation
  - phase6-ci-golden-gates
  - phase7-migration
  - phase7-docs

Commit discipline:
- Commit only related changes; no drive-by refactors.
- Use clear commits like:
  - docs: add PARITY_MATRIX baseline
  - feat(spec): add v1 schema + hashing
  - feat(cli): add validate/preview scaffolding

Validation:
- Before committing, run the narrowest relevant checks (format/lint/tests).
- If tests require tools not available in this environment, STOP and tell me exactly what you need.

PR creation:
- If `gh` CLI is available and authenticated, open a PR.
- Otherwise output a ready-to-paste PR title + description + checklist and STOP.

Execution gates:
- Do phases 0 → 7 in order. Do not skip phases.
- Before starting a phase: summarize exact deliverables and commands you will run.
- After completing a task: verify acceptance criteria, summarize results, and STOP for my approval before starting the next branch.

Start now:
- Create branch `phase0-task01-parity-matrix`.
- Implement Phase 0 Task 0.1 and produce PARITY_MATRIX.md (docs-only change).
- Commit + open/draft PR, then STOP.
```

## Sub-agent Task Tool Model Selection

Claude Code supports an optional `model` parameter for the `Task` tool:

- `model: "haiku"` — quick, straightforward tasks (recommended for exploration/search)
- `model: "sonnet"` — most coding and docs work
- `model: "opus"` — deep reasoning/architecture/tricky algorithms

If you don’t specify `model`, the sub-agent inherits the current session model.

Example:

```yaml
subagent_type: "Explore"
model: "haiku"
prompt: "Find all files that handle authentication"
```

