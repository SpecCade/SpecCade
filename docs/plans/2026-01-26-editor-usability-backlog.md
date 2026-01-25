# Editor Usability Backlog

**Goal:** Capture the highest-impact usability issues in the SpecCade editor and convert them into a prioritized backlog.

---

## Current Pain Points (Top 10)

1) **IPC failures are confusing and high-impact**
When IPC permissions are misconfigured, core workflows fail (typing/eval, open folder, save), but the UI error does not clearly point to an actionable fix. This also creates a brittle “works on my machine” situation depending on build artifacts.

2) **Stdlib discovery is fragmented (and easy to get out of sync)**
Autocomplete/snippets can diverge from the actual Rust stdlib surface, which makes the editor feel untrustworthy. Users learn by exploration in-editor, so drift is especially painful.

3) **Music composition workflows are hard to bootstrap**
The music pipeline is powerful, but the path to “first sound” requires knowing a lot of function names and structure. Missing or incomplete composer building blocks make the first iteration slow.

4) **Errors are not “actionable” enough**
Compiler/runtime errors show up, but the UI lacks consistent “jump to location” affordances and a clear separation between syntax errors vs semantic/validation errors. Iteration speed suffers.

5) **Snippets insert code, but not necessarily correct code**
If snippets are only “valid-ish”, users have to fix many small details each time. Small inconsistencies (arg names, defaults, optional args) add up and reduce confidence.

6) **Searchless palette does not scale**
Once the stdlib surface grows, a purely categorical palette becomes slow to navigate. Users want quick lookup by name (e.g. "tracker", "noise", "bevel").

7) **Templates can diverge from real workflows**
If templates do not match current best practices, they mislead new users and create churn (people copy/paste old patterns). Templates are the "first run" experience.

8) **Preview/validation latency feels opaque**
When a spec is invalid or preview generation fails, users need fast feedback and clear state transitions (pending/running/success/failure). Without good status affordances, it feels "stuck".

9) **Project navigation is minimal**
Open-folder listing helps, but common actions like recent projects, quick file search, and clear asset-type detection feedback are limited. This slows down iteration across many specs.

10) **Generated artifacts and build steps are not obvious**
Some editor behaviors depend on generated schemas/permissions and build configuration. Without documented expectations, contributors can easily break the editor or get stuck debugging environment issues.

---

## Proposed Improvements (Backlog)

1) **Make compiler errors clickable (Jump to line/column)**
What it fixes: Faster fix/verify loops when authoring specs.
Acceptance criteria: Error UI includes file + line/column when available; clicking focuses Monaco and selects the error range.
Effort: M

2) **Standardize stdlib metadata as a single manifest**
What it fixes: Drift between snippets, completions, and the Rust stdlib.
Acceptance criteria: One manifest drives both features; updating stdlib requires updating only one place; obvious tests/CI checks prevent drift.
Effort: M

3) **Searchable stdlib palette**
What it fixes: Navigation time once stdlib surface becomes large.
Acceptance criteria: Case-insensitive search across name/signature/description; hides empty categories; fast enough for "type as you search".
Effort: S

4) **Music composition quickstart template (compose-first)**
What it fixes: High friction to first audible result.
Acceptance criteria: A template produces a minimal, valid tracker song spec using composition helpers; preview playback works with minimal edits.
Effort: M

5) **Music "cue workflow" templates (loop/stinger/transition)**
What it fixes: Missing practical patterns for game music building blocks.
Acceptance criteria: Templates cover loop_low/loop_main/loop_hi/stinger/transition; each compiles and produces output with minimal edits.
Effort: M

6) **Improve status bar semantics and progress reporting**
What it fixes: Unclear "what is happening" during eval/validate/generate.
Acceptance criteria: Status shows state transitions (idle/running/success/error) and short actionable message; long operations show progress events.
Effort: M

7) **Template audit + keep templates aligned with stdlib changes**
What it fixes: Stale templates and broken onboarding.
Acceptance criteria: Templates compile/validate under strict budget; templates reference the same stdlib conventions used by completions/snippets.
Effort: M

8) **Guardrails for Tauri permissions and generated schemas**
What it fixes: Breaking IPC or capability config without noticing.
Acceptance criteria: Document the intended generation flow; add a CI check or scripted verification that capabilities/permissions are consistent.
Effort: M

9) **Add a "recent projects" list**
What it fixes: Repeated open-folder friction.
Acceptance criteria: Recently opened folders appear in a menu/panel; selecting one reopens quickly; list survives restarts.
Effort: M

10) **Add "quick open" (file search) inside the project**
What it fixes: Navigating large packs.
Acceptance criteria: A keyboard shortcut opens a search box; fuzzy matches file paths; selecting opens the file.
Effort: L

---

## Top 3 Priorities

1) **Single stdlib manifest + drift prevention** (Backlog item 2)
This directly improves trust, onboarding, and iteration speed across all asset types.

2) **Clickable, actionable errors** (Backlog item 1)
This is the highest leverage UI improvement for everyday work.

3) **Music composition quickstart + cue templates** (Backlog items 4 and 5)
This unlocks the new music workflows by making the path to "first sound" and common game-music patterns obvious.
