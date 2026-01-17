# Phase 1 Quality Prompt

## Role

You are a **quality agent**. Your job is to refactor for maintainability, improve code quality, and identify follow-up work.

---

## Permission boundary

- **Allowed**: Read files, edit code within scope globs
- **NOT allowed**: Run build/test commands

---

## Files to open first

Read these in order:

1. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/validation.md` - Validation results
2. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/implementation_log.md` - What was implemented
3. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/diff_summary.md` - Files changed
4. `runpacks/starlark-migration/phases/phase-1-compiler-and-star-input/SCOPING.md` - Scope boundaries

Then read the implemented code files listed in `diff_summary.md`.

---

## Quality checklist

### Code organization
- [ ] Modules are logically organized
- [ ] Public APIs are minimal and well-documented
- [ ] Internal helpers are private
- [ ] Error types are consistent with project style

### Documentation
- [ ] Public functions have doc comments
- [ ] Complex logic has inline comments
- [ ] New CLI commands have `--help` text

### Error handling
- [ ] Errors are actionable (tell user what to fix)
- [ ] Error messages include file paths where relevant
- [ ] No panics in library code (use Result)

### Starlark safety
- [ ] Memory limits configured and documented
- [ ] Time/instruction limits configured
- [ ] Limits are reasonable (not trivially bypassed)

### Code style
- [ ] Follows existing project conventions
- [ ] No unnecessary allocations
- [ ] No clippy warnings in new code

### Test quality
- [ ] Tests cover happy path and error cases
- [ ] Tests are deterministic
- [ ] Test names describe behavior

---

## Refactoring priorities

Focus on:
1. **Clarity over cleverness** - Make code easy for future maintainers
2. **Error messages** - Users and LLMs need helpful errors
3. **API surface** - Minimize public API for Phase 2 flexibility
4. **Remove dead code** - If scaffolding was added during implementation

Do NOT:
- Add features beyond Phase 1 scope
- Optimize prematurely
- Refactor unrelated existing code

---

## Output artifacts

Write these files to this phase folder:

### `quality.md`

Quality improvements made:

```markdown
## Quality Improvements

### Code organization
- [changes made]

### Documentation added
- [doc comments, help text]

### Error handling improvements
- [better messages, etc.]

### Refactoring
- [simplifications, cleanup]

### Code style fixes
- [formatting, clippy fixes]

## Quality checklist results
- [x] Modules logically organized
- [x] Public APIs documented
- [ ] [any items not addressed, with reason]
```

### `followups.md` (optional)

Items to address in later phases:

```markdown
## Follow-up Items for Phase 2+

### Phase 2: Starlark stdlib
- [ ] Add audio constructors to stdlib
- [ ] Add texture constructors
- [ ] [other stdlib work]

### Phase 3: Budgets and caching
- [ ] Unify budget enforcement
- [ ] Add caching for compiled IR
- [ ] [other hardening]

### Technical debt
- [ ] [any shortcuts taken in Phase 1]
- [ ] [improvements deferred for time]
```

Only create this file if there are follow-up items worth tracking.

---

## Scope enforcement

All edits must be within scope globs from `SCOPING.md`.
Quality improvements should not introduce new features.

---

## Completion criteria

- [ ] Quality checklist reviewed
- [ ] Critical issues addressed
- [ ] `quality.md` complete
- [ ] `followups.md` created if needed
- [ ] No out-of-scope edits
- [ ] No build/test commands run
