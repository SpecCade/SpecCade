# Editor Problems + Music/Texture Inspectors Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an actionable Problems view (click-to-jump), a tracker-focused Music order/pattern navigator, and a Texture inspector (seam + channel + mip views) while staying code-first.

**Architecture:** Keep Starlark as the single source of truth. Add read-only inspector UIs driven by existing backend outputs and `chiptune3` progress events. Problems are gathered from the existing eval/preview pipeline and rendered in the editor as a third tab alongside Preview/Generate.

**Tech Stack:** Tauri 2, TypeScript, Monaco, `chiptune3`, Canvas 2D.

---

## Workspace Setup (One-Time)

**Step 1: Create a worktree**

Run:
```bash
git worktree add ".worktrees/editor-music-order-inspector" -b "editor-music-order-inspector"
```

**Step 2: Install editor deps**

Run:
```bash
npm -C editor ci
```

**Step 3: Verify baseline**

Run:
```bash
cargo test -p tauri-plugin-speccade
npm -C editor run typecheck
```
Expected: both pass.

---

## Task 1: Add Problems Tab + Panel Skeleton

**Files:**
- Modify: `editor/index.html`
- Create: `editor/src/components/ProblemsPanel.ts`
- Modify: `editor/src/main.ts`

**Step 1: Add a Problems tab and container**

In `editor/index.html`, extend the preview header tab row:
- Add a third button: `problems-tab`
- Add a third content container: `problems-content` (initially `display: none`)

**Step 2: Create `ProblemsPanel` component**

Create `editor/src/components/ProblemsPanel.ts`:
- Render a header row with counts (Errors, Warnings)
- Render an empty state when there are no problems
- Expose:
  - `setProblems(problems: ProblemItem[]): void`
  - `clear(): void`
  - `dispose(): void`

Suggested types:
```ts
export type ProblemStage = "eval" | "compile" | "preview" | "ipc";

export type ProblemItem = {
  id: string;
  stage: ProblemStage;
  severity: "error" | "warning";
  code?: string;
  message: string;
  location?: { line: number; column: number };
};
```

Also accept an `onSelect(problem)` callback in the constructor.

**Step 3: Wire tab switching for 3 tabs**

In `editor/src/main.ts`, replace the 2-tab logic with a helper that:
- Shows exactly one of `preview-content`, `generate-content`, `problems-content`
- Updates tab button styles consistently

**Step 4: Typecheck**

Run:
```bash
npm -C editor run typecheck
```
Expected: pass.

---

## Task 2: Populate Problems From `eval_spec` (Click To Jump)

**Files:**
- Modify: `editor/src/main.ts`
- Modify: `editor/src/components/Editor.ts`

**Step 1: Expose selection helper on `Editor`**

In `editor/src/components/Editor.ts`, add:
```ts
selectAt(line: number, column: number): void
```

**Step 2: Wire ProblemsPanel to the editor**

In `editor/src/main.ts`:
- Instantiate `ProblemsPanel` in `init()`
- Provide `onSelect` that calls `editor.selectAt(...)` when a location exists

**Step 3: Track problems in `evaluateSource`**

Update `evaluateSource` to:
- Convert `EvalError[]` and `EvalWarning[]` into `ProblemItem[]` with stage `eval`
- Keep using Monaco markers via `editor.setDiagnostics(...)`
- Call `problemsPanel.setProblems(...)` every evaluation (including success/warnings-only)

---

## Task 3: Include Preview/Compile/IPC Errors In Problems

**Files:**
- Modify: `editor/src/main.ts`

**Step 1: Add helpers to update problems from preview pipeline**

In `editor/src/main.ts`, maintain a single source of truth for problems and add `replaceStage(stage, items)`.

**Step 2: Capture `compile_error`**

In `renderAudioPreview`, `renderMusicPreview` (requestPreview path), `renderTexturePreview`, `renderMeshPreview`:
- When `!compile_success`, set stage `compile` severity `error` from `compile_error`.

**Step 3: Capture `preview.error`**

When `preview?.success` is false, set stage `preview` severity `error` from `preview.error`.

**Step 4: Capture IPC errors**

If IPC fails (invoke throws), set stage `ipc` severity `error`.

---

## Task 4: Music Order/Pattern Navigator (Best-Effort)

**Files:**
- Modify: `editor/src/components/MusicPreview.ts`

Add:
- Now Playing line: `Ord -- | Pat -- | Row --`
- Order strip: clickable order chips that populate as playback progresses

Best-effort mapping is built from `chiptune3` progress events.

---

## Task 5: Texture Inspector (Seam + Channel + Mip)

**Files:**
- Modify: `editor/src/components/TexturePreview.ts`

Add controls:
- Seam toggle: centered 3x3 tiling + border lines
- Channel selector: RGB, R, G, B, A
- Mip selector: 0..N using cached downscaled offscreen canvases

---

## Task 6: Manual Verification + Build

Run:
```bash
npm -C editor run build
npm -C editor run tauri dev
```

Manual checks:
- Problems tab shows errors/warnings; clicking jumps to the right line.
- Music preview: order strip fills in during playback; clicking a discovered order seeks.
- Texture preview: seam/channel/mip controls work and are clearly visible.
