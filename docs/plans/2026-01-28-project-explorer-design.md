# ProjectExplorer — GUI Usability Redesign

**Date:** 2026-01-28
**Status:** Design approved, not yet implemented

## Summary

Replace the existing `FileBrowser` component with a new `ProjectExplorer` that adds: tree view, asset type filters, fuzzy search, quick-open (Ctrl+P), multi-select with batch operations, and full keyboard navigation.

## Component Architecture

```
ProjectExplorer (coordinator)
├── SearchBar          — fuzzy filename search + Ctrl+P quick-open overlay
├── FilterBar          — asset type toggle chips (audio/music/texture/mesh/all)
├── FileTree           — expandable folder tree with virtual scrolling
└── SelectionManager   — tracks multi-select state, exposes batch actions
```

ProjectExplorer owns the file data model: a recursive `FileNode[]` tree built from a new backend command `scan_project_tree` that returns the full directory structure in one call. It caches this tree and refreshes on file-watcher events.

**Data flow:** `scan_project_tree` → ProjectExplorer stores tree → SearchBar/FilterBar produce filter predicates → FileTree renders filtered view → SelectionManager tracks checked items.

## FileTree

Renders a flat list of visible nodes (expanded folders + their visible children), not a nested DOM tree.

### FileNode Interface

```ts
interface FileNode {
  name: string;
  path: string;           // relative to project root
  isDir: boolean;
  assetType?: string;     // "audio" | "music" | "texture" | "mesh" | "skeletal_mesh" | "animation"
  children?: FileNode[];
  expanded: boolean;
  depth: number;
  selected: boolean;
}
```

### Rendering

Flatten the tree into a visible-rows array (only expanded branches). Each row is a `<div>` with `padding-left: depth * 16px`. Folders get a ▶/▼ toggle. Files get emoji icons by asset type. The active file (open in editor) is highlighted.

### Virtual Scrolling

Only render rows visible in the scroll viewport plus a small buffer. Future-proofs for large projects.

### Keyboard Navigation

- Arrow keys: move focus cursor through visible rows
- Enter: open file / toggle folder
- Space: toggle selection
- Home/End: jump to first/last
- Type-ahead: typing characters jumps to matching filename

## FilterBar

Sits below the search bar. Displays toggle chips for each asset type:

```
[All] [🔊 Audio] [🎵 Music] [🎨 Texture] [📦 Mesh] [🧍 Skel] [🏃 Anim]
```

- Click a chip to show only that type. Click again to deselect (back to All).
- Multiple chips can be active simultaneously.
- "All" is the default and deselects when any specific type is chosen.
- Folders remain visible if they contain matching descendants.
- Count badges show matches per type: `[🔊 Audio (12)]`

## SearchBar

Text input at the top of the sidebar:

- Filters the tree in real-time (debounced 150ms).
- Fuzzy match against filename — highlights matched characters.
- When active, auto-expands folders containing matches, hides non-matching branches.
- Combines with FilterBar (both predicates apply).
- Clear button (×) and Escape to reset.

## Quick Open (Ctrl+P)

Centered overlay dialog (VS Code-style command palette):

- Fuzzy search across entire project.
- Arrow keys to navigate results, Enter to open.
- Separate from sidebar search — keyboard-driven jump-to-file.

## Multi-Select & Batch Operations

### Selection Controls

- Checkbox on each file row (on hover, or always in multi-select mode).
- Shift+Click: select range.
- Ctrl+A: select all visible (filtered) files.
- Escape: clear selection.
- "Select All / Deselect All" toggle in header.

### Batch Action Bar

When 1+ files selected, a sticky bar appears at the sidebar bottom:

```
┌─────────────────────────────┐
│ 5 selected                  │
│ [Validate] [Generate] [Del] │
└─────────────────────────────┘
```

- **Validate:** Run validation on all selected, aggregate results in Problems panel.
- **Generate:** Batch-generate assets (uses existing `batch_generate` backend command).
- **Delete:** Delete with confirmation dialog.

## Backend Changes

### New Command: `scan_project_tree`

```rust
#[tauri::command]
async fn scan_project_tree(path: String) -> Result<Vec<FileNode>, String>
```

Recursively walks project directory. Returns full tree with asset type detection. Filters hidden files/dirs and non-spec files (`.star`, `.json` only).

### File Watcher Integration

Existing `watcher.rs` events trigger incremental updates — re-scan only the changed subtree.

No other backend changes needed. Validation, generation, and batch commands already exist.

## Implementation Order

1. `scan_project_tree` backend command
2. FileTree component (tree view with expand/collapse)
3. FilterBar (asset type chips)
4. SearchBar (fuzzy filename filter)
5. Quick Open dialog (Ctrl+P overlay)
6. SelectionManager + batch action bar
7. Keyboard navigation
8. Wire up & replace FileBrowser in main.ts

Steps 2–4 are core value. Steps 5–7 are polish. Each step is independently testable.

## Tech Decisions

- Pure TypeScript components (no framework).
- Follows existing component patterns in the codebase.
- Existing FileBrowser stays until full replacement is wired up.
