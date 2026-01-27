# Editor Music Playback (XM/IT) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add in-editor preview playback for SpecCade `music` assets by generating XM/IT modules on demand and playing them in the preview pane.

**Architecture:**
1) Add a new Rust preview backend (`speccade-editor` crate) that returns the generated `.xm`/`.it` bytes as base64. 2) Add a new frontend `MusicPreview` component powered by `chiptune3` (libopenmpt AudioWorklet) to play those bytes. 3) Default to on-demand generation, with a user toggle to enable “live” regeneration on debounced edits.

**Tech Stack:** Rust (speccade-editor preview backends), Tauri IPC, TypeScript/DOM UI, `chiptune3` (libopenmpt), Web Audio API.

---

## Non-Goals (for this plan)

- Mixdown to WAV, spectrograms, piano-roll/pattern-grid editor UI
- Instrument audition UI (per-instrument solo)
- Advanced transport sync with pattern/row highlighting (beyond showing basic progress metadata)

---

## Task 1: Add Music Preview Backend (Rust)

**Files:**
- Create: `crates/speccade-editor/src/preview/music.rs`
- Modify: `crates/speccade-editor/src/preview/mod.rs`

**Step 1: Add failing tests for the new preview module**

Create `crates/speccade-editor/src/preview/music.rs` with a stub `generate_music_preview(...)` that returns a failure. Add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec};
    use speccade_spec::recipe::music::{
        ArrangementEntry, InstrumentSynthesis, MusicTrackerSongV1Params, PatternNote,
        TrackerFormat, TrackerInstrument, TrackerPattern,
    };
    use std::collections::HashMap;

    #[test]
    fn test_music_preview_no_recipe() {
        let spec = Spec::builder("test-music", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "music/test.xm"))
            .build();

        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.star");
        let result = generate_music_preview(&spec, &spec_path);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_music_preview_wrong_recipe_type() {
        let recipe = Recipe::new("audio_v1", serde_json::json!({}));
        let spec = Spec::builder("test-music", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "music/test.xm"))
            .recipe(recipe)
            .build();

        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.star");
        let result = generate_music_preview(&spec, &spec_path);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not a music"));
    }

    #[test]
    fn test_music_preview_generates_module_bytes() {
        // Minimal 1-pattern XM.
        let instrument = TrackerInstrument {
            name: "lead".to_string(),
            synthesis: Some(InstrumentSynthesis::Square),
            ..Default::default()
        };

        let note = PatternNote {
            row: 0,
            note: "C4".to_string(),
            inst: 0,
            ..Default::default()
        };

        let mut notes_map = HashMap::new();
        notes_map.insert("0".to_string(), vec![note]);

        let pattern = TrackerPattern {
            rows: 64,
            notes: Some(notes_map),
            data: None,
        };

        let mut patterns = HashMap::new();
        patterns.insert("p0".to_string(), pattern);

        let params = MusicTrackerSongV1Params {
            format: TrackerFormat::Xm,
            bpm: 120,
            speed: 6,
            channels: 4,
            instruments: vec![instrument],
            patterns,
            arrangement: vec![ArrangementEntry { pattern: "p0".to_string(), repeat: 1 }],
            ..Default::default()
        };

        let recipe = Recipe::new(
            "music.tracker_song_v1",
            serde_json::to_value(params).unwrap(),
        );

        let spec = Spec::builder("test-music", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "music/test.xm"))
            .recipe(recipe)
            .build();

        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.star");
        let result = generate_music_preview(&spec, &spec_path);
        assert!(result.success);
        assert!(result.data.is_some());
        assert_eq!(result.asset_type, "music");
    }
}
```

**Step 2: Run the tests to confirm they fail**

Run: `cargo test -p tauri-plugin-speccade music_preview`

Expected: FAIL (function not implemented / returns failure)

**Step 3: Implement `generate_music_preview`**

Implement generation by:
- validating recipe presence and that `recipe.kind` starts with `music.`
- creating a temp output dir
- calling `speccade_cli::dispatch::dispatch_generate(spec, tmp_out, spec_path, None)`
- selecting the first primary output with format `xm` or `it`
- reading bytes back from temp output dir and returning `PreviewResult::success_with_metadata("music", bytes, mime, metadata)`

**Step 4: Run tests to verify pass**

Run: `cargo test -p tauri-plugin-speccade music_preview`

Expected: PASS

**Step 5: (Optional) Commit**

```bash
git add crates/speccade-editor/src/preview/mod.rs crates/speccade-editor/src/preview/music.rs
git commit -m "feat(editor): add music preview backend returning XM/IT bytes"
```

---

## Task 2: Route `AssetType::Music` Preview to the New Backend (Rust)

**Files:**
- Modify: `crates/speccade-editor/src/commands/generate.rs`

**Step 1: Add a failing test that proves music preview works via `generate_preview`**

Add a new unit test in `crates/speccade-editor/src/commands/generate.rs` that calls `generate_preview(...)` on a minimal music spec with a `music.tracker_song_v1` recipe and asserts `preview.success == true`.

**Step 2: Run the test to confirm it fails**

Run: `cargo test -p tauri-plugin-speccade generate_preview_music`

Expected: FAIL (until routing is fixed)

**Step 3: Update routing + spec_dir behavior**

In the `match spec.asset_type` inside `generate_preview`, change:

- `AssetType::Music` to call `preview::music::generate_music_preview(&spec, Path::new(&filename))`

This ensures relative paths in music instruments (`wav`, `ref`) resolve relative to the currently open spec file.

**Step 4: Run Rust tests**

Run: `cargo test -p tauri-plugin-speccade`

Expected: PASS

**Step 5: (Optional) Commit**

```bash
git add crates/speccade-editor/src/commands/generate.rs
git commit -m "fix(editor): enable music preview routing in generate_preview"
```

---

## Task 3: Add `chiptune3` + MusicPreview Component (Frontend)

**Files:**
- Modify: `editor/package.json`
- Create: `editor/src/components/MusicPreview.ts`

**Step 1: Add `chiptune3` dependency**

Run:

`cd editor && npm install chiptune3`

**Step 2: Create `MusicPreview` component**

Create `editor/src/components/MusicPreview.ts`:

- Uses `ChiptuneJsPlayer` from `chiptune3`
- Provides UI:
  - Play/Pause, Stop, Refresh (regenerate)
  - Seek slider + volume slider
  - “Live preview” checkbox (stored in `localStorage`) that toggles between:
    - on-demand generation
    - regenerate on debounced editor edits (driven by `main.ts` calling `musicPreview.setSource(...)`)
- Exposes methods:
  - `setSource(source: string, filename: string): void`
  - `setPreviewBytes(base64: string): Promise<void>` (decodes base64 -> ArrayBuffer -> `player.play()`)
  - `dispose(): void`

**Step 3: Verify TS typecheck**

Run: `cd editor && npm run typecheck`

Expected: PASS

**Step 4: (Optional) Commit**

```bash
git add editor/package.json editor/package-lock.json editor/src/components/MusicPreview.ts
git commit -m "feat(editor): add MusicPreview player via chiptune3"
```

---

## Task 4: Integrate MusicPreview + Fix Asset Type Routing (Frontend)

**Files:**
- Modify: `editor/src/main.ts`
- Modify: `editor/src/components/GeneratePanel.ts`

**Step 1: Fix preview routing for canonical asset types**

Update `updatePreview(...)` in `editor/src/main.ts` to map:

- `audio` -> audio preview
- `music` -> music preview
- `texture` / `sprite` / `ui` / `font` / `vfx` -> texture preview
- `static_mesh` / `skeletal_mesh` -> mesh preview

**Step 2: Pass real file path as `filename` to backend**

Update all IPC calls that pass `filename: "editor.star"` to instead use:

`filename: currentFilePath ?? "editor.star"`

At minimum:
- `eval_spec`
- `generate_preview` (for mesh/audio/texture)

This is required for music instrument refs (`wav` / `ref`) to resolve relative to the open file.

**Step 3: Wire MusicPreview on-demand + live toggle**

- Create `musicPreview` state similar to `audioPreview`.
- When `asset_type === "music"`:
  - instantiate `MusicPreview` (if needed)
  - call `musicPreview.setSource(source, currentFilePath ?? "editor.star")`
  - provide a callback to `MusicPreview` so it can request regeneration:
    - invoke `plugin:speccade|generate_preview`
    - on success: `musicPreview.setPreviewBytes(preview.data)`

**Step 4: Improve GeneratePanel to use real filename**

Update `editor/src/components/GeneratePanel.ts` to accept a `getFilename: () => string` callback (like `getSource`). Use it when calling `generate_full`.

**Step 5: Verify TS build**

Run: `cd editor && npm run build`

Expected: PASS

**Step 6: (Optional) Commit**

```bash
git add editor/src/main.ts editor/src/components/GeneratePanel.ts
git commit -m "feat(editor): add music playback preview and correct asset routing"
```

---

## Task 5: End-to-End Verification

**Step 1: Run Rust tests**

Run: `cargo test -p tauri-plugin-speccade`

Expected: PASS

**Step 2: Run editor dev build**

Run: `cd editor && npm run build`

Expected: PASS

**Step 3: Manual test (Tauri dev)**

Run: `cd editor && npm run tauri dev`

Manual checks:
- Open a music spec (or create new from template) and confirm preview pane shows MusicPreview controls.
- Click Play: should generate module and play.
- Toggle “Live preview”: edit the spec, wait for debounce, then Refresh should happen automatically.
- If the music instrument uses `wav` or `ref` paths relative to the spec file, ensure playback still works when file is opened from the File Browser.
