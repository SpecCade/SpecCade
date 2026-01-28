#!/usr/bin/env markdown

# Mesh Preview Material Layer (Lit / Unlit / Matcap)

Status: Draft (brainstorm validated)

## Goal

Improve the editor 3D mesh preview so users can quickly read form and textures without building a full material editor.

Add a lightweight, swappable render/material layer:

- `Lit (glTF)` - current behavior
- `Unlit (Texture)` - UV texture shown without lighting
- `Matcap` - matcap shading for fast form readability

The feature must stay small and non-opinionated:

- No multi-map PBR wiring
- No shader graph UI
- One optional texture slot only
- Never mutates the user's spec; preview-only

## UX

In the mesh preview overlay (`editor/src/components/MeshPreview.ts`):

- `Render:` select: `Lit`, `Unlit`, `Matcap`
- `Texture:` select:
  - `None`
  - `Golden: ...` (curated list)
  - `From PNG file...`
  - `From spec output...`

Rules:

- The texture slot is PNG-agnostic.
  - In `Unlit`, interpret it as a 2D UV texture (`map`, UV0).
  - In `Matcap`, interpret it as a matcap (`matcap`, ignores UVs).
- Any PNG can be used in any mode; no hard validation.
- Persist user choices globally via `localStorage`.

## Texture Sources

### 1) Golden specs (dogfood)

Use curated "production ready" golden specs from `golden/starlark/`.

Important: packaged builds must not require repo checkout.

Approach:

- Embed golden spec sources into the Tauri plugin via `include_str!`.
- Expose them via IPC.
- Still generate the PNG through the real pipeline (`plugin:speccade|generate_preview`) so the editor dogfoods the same code paths.

### 2) Existing PNG file on disk

Use file picker -> read bytes via IPC -> apply as texture slot.

### 3) Output declared in a spec

Use file picker to choose a spec file, then choose a specific declared PNG output.

Implementation:

- Read spec source (text) via existing `plugin:speccade|read_file`.
- Use `plugin:speccade|eval_spec` to list `outputs[]` and filter to PNG.
- Call a dedicated IPC command that:
  - compiles the spec,
  - validates that the selected output path exists and is PNG,
  - runs generation into a temp directory,
  - reads back exactly that output and returns it as base64.

## State Persistence

Store a single global state blob:

- key: `speccade:mesh_preview_material:v1`
- shape:
  - `render_mode: "lit" | "unlit" | "matcap"`
  - `texture_ref: null | { kind, ... }`
    - `golden`: `{ kind: "golden", id }`
    - `file`: `{ kind: "file", path }`
    - `spec_output`: `{ kind: "spec_output", spec_path, output_path }`

Bytes are not persisted.

## Backend IPC (crates/speccade-editor)

Add commands:

- `list_golden_preview_textures() -> [{ id, label, kind }]`
- `get_golden_preview_texture_source(id) -> { filename, source }`
- `read_binary_file_base64(path) -> { base64, mime_type }`
- `generate_png_output_base64(source, filename, output_path) -> { base64, mime_type }`

Update Tauri command registration + permissions:

- `crates/speccade-editor/src/lib.rs`
- `crates/speccade-editor/build.rs`
- `crates/speccade-editor/permissions/default.toml`

## Frontend (editor)

`editor/src/components/MeshPreview.ts`:

- Add overlay controls.
- Maintain a cached copy of the original GLTF materials and restore them for `Lit`.
- For `Unlit` and `Matcap`, swap materials on each mesh and dispose override materials on mode changes.
- Load texture slot data by source:
  - Golden: `get_golden_preview_texture_source` -> `generate_preview`
  - File: `read_binary_file_base64`
  - Spec output: read+eval -> `generate_png_output_base64`

## Non-Goals / Future Work

- Multi-map PBR preview (albedo+normal+roughness)
- Preset shaders ("nethercore-zx", etc.)
- Per-file state
- Automatic binding of textures to mesh materials
