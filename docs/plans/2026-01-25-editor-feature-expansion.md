# Editor Feature Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform the SpecCade editor from a single-file text editor into a complete asset authoring workspace with asset type wizards, project management, export capabilities, and discoverability features.

**Architecture:** Extend the existing Tauri/Monaco editor with: (1) sidebar navigation for project files and asset browser, (2) asset creation wizards using templates, (3) generate/export panel with batch support, (4) stdlib palette and snippet insertion. All changes build on the existing `speccade-editor` crate and TypeScript frontend.

**Tech Stack:** Tauri 2.x, TypeScript, Monaco Editor, three.js, Web Audio API, speccade-cli integration

---

## Overview

The plan is organized into four phases addressing the four gaps:

1. **Asset Type Workflows** (Tasks 1-4) - Wizards for sounds, music, textures, meshes
2. **Project Management** (Tasks 5-8) - Multi-file project explorer, recent files, save/load
3. **Export & Generation** (Tasks 9-12) - Full asset export, batch generation, pack manifests
4. **Discoverability** (Tasks 13-16) - Templates palette, snippet insertion, help panel

Each task is designed to be independently testable and committable.

---

## Phase 1: Asset Type Workflows

### Task 1: Add Asset Type Selector Panel

**Files:**
- Create: `editor/src/components/AssetTypeSelector.ts`
- Modify: `editor/src/main.ts`
- Modify: `editor/index.html`

**Step 1: Create the AssetTypeSelector component**

Create `editor/src/components/AssetTypeSelector.ts`:

```typescript
/**
 * Asset type selector panel for creating new specs.
 */

export interface AssetTypeInfo {
  id: string;
  name: string;
  icon: string;
  description: string;
  template: string;
}

const ASSET_TYPES: AssetTypeInfo[] = [
  {
    id: "audio",
    name: "Sound Effect",
    icon: "🔊",
    description: "Procedural audio synthesis (lasers, explosions, UI sounds)",
    template: "audio_basic",
  },
  {
    id: "music",
    name: "Music Track",
    icon: "🎵",
    description: "Tracker-style music composition (XM/IT format)",
    template: "music_basic",
  },
  {
    id: "texture",
    name: "Texture",
    icon: "🎨",
    description: "Procedural textures (noise, patterns, PBR materials)",
    template: "texture_basic",
  },
  {
    id: "static_mesh",
    name: "3D Mesh",
    icon: "📦",
    description: "Static meshes from primitives with modifiers",
    template: "mesh_basic",
  },
  {
    id: "skeletal_mesh",
    name: "Character",
    icon: "🧍",
    description: "Rigged skeletal meshes with body parts",
    template: "character_basic",
  },
  {
    id: "skeletal_animation",
    name: "Animation",
    icon: "🏃",
    description: "Skeletal animations with keyframes",
    template: "animation_basic",
  },
  {
    id: "sprite",
    name: "Sprite Sheet",
    icon: "🖼️",
    description: "2D sprite sheets and animations",
    template: "sprite_basic",
  },
  {
    id: "vfx",
    name: "VFX",
    icon: "✨",
    description: "Visual effects flipbooks and particles",
    template: "vfx_basic",
  },
];

export class AssetTypeSelector {
  private container: HTMLElement;
  private onSelect: (template: string) => void;

  constructor(container: HTMLElement, onSelect: (template: string) => void) {
    this.container = container;
    this.onSelect = onSelect;
    this.render();
  }

  private render(): void {
    const wrapper = document.createElement("div");
    wrapper.className = "asset-type-selector";
    wrapper.style.cssText = `
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 8px;
      padding: 12px;
    `;

    for (const assetType of ASSET_TYPES) {
      const card = this.createCard(assetType);
      wrapper.appendChild(card);
    }

    this.container.appendChild(wrapper);
  }

  private createCard(info: AssetTypeInfo): HTMLElement {
    const card = document.createElement("button");
    card.className = "asset-type-card";
    card.style.cssText = `
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 12px;
      background: #2a2a2a;
      border: 1px solid #444;
      border-radius: 6px;
      cursor: pointer;
      transition: all 0.15s;
      text-align: center;
    `;

    card.innerHTML = `
      <span style="font-size: 24px; margin-bottom: 4px;">${info.icon}</span>
      <span style="font-size: 12px; font-weight: 500; color: #fff;">${info.name}</span>
      <span style="font-size: 10px; color: #888; margin-top: 4px;">${info.description}</span>
    `;

    card.addEventListener("mouseenter", () => {
      card.style.background = "#333";
      card.style.borderColor = "#007acc";
    });

    card.addEventListener("mouseleave", () => {
      card.style.background = "#2a2a2a";
      card.style.borderColor = "#444";
    });

    card.addEventListener("click", () => {
      this.onSelect(info.template);
    });

    return card;
  }

  dispose(): void {
    this.container.innerHTML = "";
  }
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add editor/src/components/AssetTypeSelector.ts
git commit -m "feat(editor): add AssetTypeSelector component for new asset creation"
```

---

### Task 2: Add Template Loading Backend

**Files:**
- Create: `crates/speccade-editor/src/commands/templates.rs`
- Modify: `crates/speccade-editor/src/commands/mod.rs`
- Modify: `crates/speccade-editor/src/lib.rs`

**Step 1: Create templates command**

Create `crates/speccade-editor/src/commands/templates.rs`:

```rust
//! Template loading commands for the editor.

use serde::{Deserialize, Serialize};

/// Available template info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: String,
}

/// Get the list of available templates.
#[tauri::command]
pub fn list_templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            id: "audio_basic".into(),
            name: "Basic Sound Effect".into(),
            description: "Simple oscillator with envelope".into(),
            asset_type: "audio".into(),
        },
        TemplateInfo {
            id: "audio_laser".into(),
            name: "Laser Shot".into(),
            description: "Sci-fi laser sound with pitch sweep".into(),
            asset_type: "audio".into(),
        },
        TemplateInfo {
            id: "music_basic".into(),
            name: "Basic Track".into(),
            description: "4-channel tracker song template".into(),
            asset_type: "music".into(),
        },
        TemplateInfo {
            id: "texture_basic".into(),
            name: "Basic Noise Texture".into(),
            description: "Perlin noise with color ramp".into(),
            asset_type: "texture".into(),
        },
        TemplateInfo {
            id: "texture_pbr".into(),
            name: "PBR Material".into(),
            description: "Full PBR texture set (albedo, normal, roughness, metallic)".into(),
            asset_type: "texture".into(),
        },
        TemplateInfo {
            id: "mesh_basic".into(),
            name: "Basic Cube".into(),
            description: "Beveled cube with subdivision".into(),
            asset_type: "static_mesh".into(),
        },
        TemplateInfo {
            id: "character_basic".into(),
            name: "Basic Humanoid".into(),
            description: "Simple humanoid rig with body parts".into(),
            asset_type: "skeletal_mesh".into(),
        },
        TemplateInfo {
            id: "animation_basic".into(),
            name: "Walk Cycle".into(),
            description: "Basic walk animation for humanoid".into(),
            asset_type: "skeletal_animation".into(),
        },
        TemplateInfo {
            id: "sprite_basic".into(),
            name: "Sprite Sheet".into(),
            description: "Basic animated sprite sheet".into(),
            asset_type: "sprite".into(),
        },
        TemplateInfo {
            id: "vfx_basic".into(),
            name: "Particle Effect".into(),
            description: "Basic VFX flipbook".into(),
            asset_type: "vfx".into(),
        },
    ]
}

/// Get template content by ID.
#[tauri::command]
pub fn get_template(id: String) -> Result<String, String> {
    match id.as_str() {
        "audio_basic" => Ok(AUDIO_BASIC_TEMPLATE.to_string()),
        "audio_laser" => Ok(AUDIO_LASER_TEMPLATE.to_string()),
        "music_basic" => Ok(MUSIC_BASIC_TEMPLATE.to_string()),
        "texture_basic" => Ok(TEXTURE_BASIC_TEMPLATE.to_string()),
        "texture_pbr" => Ok(TEXTURE_PBR_TEMPLATE.to_string()),
        "mesh_basic" => Ok(MESH_BASIC_TEMPLATE.to_string()),
        "character_basic" => Ok(CHARACTER_BASIC_TEMPLATE.to_string()),
        "animation_basic" => Ok(ANIMATION_BASIC_TEMPLATE.to_string()),
        "sprite_basic" => Ok(SPRITE_BASIC_TEMPLATE.to_string()),
        "vfx_basic" => Ok(VFX_BASIC_TEMPLATE.to_string()),
        _ => Err(format!("Unknown template: {}", id)),
    }
}

const AUDIO_BASIC_TEMPLATE: &str = r#"# Basic Sound Effect
# A simple oscillator with envelope

spec(
    asset_id = "my-sound",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/my-sound.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    oscillator(440, "sine"),
                    envelope(0.01, 0.1, 0.5, 0.2)
                )
            ]
        }
    }
)
"#;

const AUDIO_LASER_TEMPLATE: &str = r#"# Laser Shot Sound
# Sci-fi laser with pitch sweep and reverb

spec(
    asset_id = "laser-shot",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/laser.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.3,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    oscillator(880, "sawtooth", 220, "exponential"),
                    envelope(0.01, 0.05, 0.0, 0.15),
                    filter = lowpass(3000, 0.707, 500)
                )
            ],
            "effects": [reverb(0.2, 0.3)]
        }
    }
)
"#;

const MUSIC_BASIC_TEMPLATE: &str = r#"# Basic Music Track
# 4-channel tracker song template

music_spec(
    asset_id = "my-track",
    seed = 42,
    output_path = "music/track.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [
        tracker_instrument(
            name = "lead",
            synthesis = instrument_synthesis("square", duty_cycle = 0.5)
        ),
        tracker_instrument(
            name = "bass",
            synthesis = instrument_synthesis("sawtooth")
        )
    ],
    patterns = {
        "intro": tracker_pattern(64, notes = {
            "0": [
                pattern_note(0, "C4", 0),
                pattern_note(16, "E4", 0),
                pattern_note(32, "G4", 0),
                pattern_note(48, "C5", 0)
            ],
            "1": [
                pattern_note(0, "C2", 1),
                pattern_note(32, "G2", 1)
            ]
        })
    },
    arrangement = [arrangement_entry("intro", 4)]
)
"#;

const TEXTURE_BASIC_TEMPLATE: &str = r#"# Basic Noise Texture
# Perlin noise with color ramp

spec(
    asset_id = "noise-tex",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/noise.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                noise_node("base", "perlin", 0.05, 4),
                color_ramp_node("colored", "base", ["#1a1a2e", "#4a4e69", "#9a8c98"])
            ]
        )
    }
)
"#;

const TEXTURE_PBR_TEMPLATE: &str = r#"# PBR Material Texture Set
# Full PBR texture set with albedo, normal, roughness, metallic

spec(
    asset_id = "metal-pbr",
    asset_type = "texture",
    seed = 42,
    outputs = [
        output("textures/metal_albedo.png", "png"),
        output("textures/metal_normal.png", "png"),
        output("textures/metal_roughness.png", "png"),
        output("textures/metal_metallic.png", "png")
    ],
    recipe = material_preset_v1(
        preset = "BrushedMetal",
        size = [512, 512],
        scale = 2.0,
        tint = [0.8, 0.8, 0.85, 1.0]
    )
)
"#;

const MESH_BASIC_TEMPLATE: &str = r#"# Basic Mesh
# Beveled cube with subdivision

spec(
    asset_id = "my-cube",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [1.0, 1.0, 1.0],
            [bevel_modifier(0.05, 2), subdivision_modifier(1)]
        )
    }
)
"#;

const CHARACTER_BASIC_TEMPLATE: &str = r#"# Basic Humanoid Character
# Simple humanoid with basic body parts

skeletal_mesh_spec(
    asset_id = "humanoid",
    seed = 42,
    output_path = "characters/humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    body_parts = [
        body_part(
            bone = "chest",
            primitive = "cylinder",
            dimensions = [0.3, 0.3, 0.28],
            segments = 8,
            material_index = 0
        ),
        body_part(
            bone = "head",
            primitive = "sphere",
            dimensions = [0.12, 0.15, 0.12],
            segments = 12,
            material_index = 0
        )
    ],
    material_slots = [
        material_slot(name = "body", base_color = [0.8, 0.6, 0.5, 1.0])
    ],
    skinning = skinning_config(max_bone_influences = 4)
)
"#;

const ANIMATION_BASIC_TEMPLATE: &str = r#"# Walk Cycle Animation
# Basic walk animation for humanoid

skeletal_animation_spec(
    asset_id = "walk-cycle",
    seed = 42,
    output_path = "animations/walk.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "walk",
    duration_seconds = 1.0,
    fps = 24,
    loop = True,
    keyframes = [
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-25.0, 0.0, 0.0])
            }
        ),
        animation_keyframe(
            time = 0.5,
            bones = {
                "upper_leg_l": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [25.0, 0.0, 0.0])
            }
        )
    ],
    interpolation = "linear"
)
"#;

const SPRITE_BASIC_TEMPLATE: &str = r#"# Sprite Sheet
# Basic animated sprite sheet

spec(
    asset_id = "sprite-sheet",
    asset_type = "sprite",
    seed = 42,
    outputs = [
        output("sprites/sheet.png", "png"),
        output("sprites/sheet.json", "json")
    ],
    recipe = {
        "kind": "sprite.sheet_v1",
        "params": {
            "frame_size": [32, 32],
            "frames": 4,
            "animation": {
                "name": "idle",
                "fps": 8,
                "loop": true
            }
        }
    }
)
"#;

const VFX_BASIC_TEMPLATE: &str = r#"# VFX Particle Effect
# Basic additive particle flipbook

spec(
    asset_id = "particle-fx",
    asset_type = "vfx",
    seed = 42,
    outputs = [
        output("vfx/particles.png", "png"),
        output("vfx/particles.json", "json")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "frame_size": [64, 64],
            "frames": 8,
            "profile": "additive",
            "color_gradient": ["#ffffff", "#ffaa00", "#ff4400", "#00000000"]
        }
    }
)
"#;
```

**Step 2: Register module in commands/mod.rs**

In `crates/speccade-editor/src/commands/mod.rs`, add:

```rust
pub mod templates;
```

**Step 3: Register commands in lib.rs**

In `crates/speccade-editor/src/lib.rs`, add imports:

```rust
pub use commands::templates::{get_template, list_templates};
```

And add to the invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    eval_spec,
    validate_spec,
    generate_preview,
    refine_mesh_preview,
    watch_file,
    unwatch_file,
    list_templates,
    get_template,
])
```

**Step 4: Run tests**

Run: `cd /c/Development/nethercore-project/speccade && cargo test -p tauri-plugin-speccade`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/speccade-editor/
git commit -m "feat(editor): add template loading backend with 10 asset templates"
```

---

### Task 3: Integrate New Asset Dialog

**Files:**
- Create: `editor/src/components/NewAssetDialog.ts`
- Modify: `editor/src/main.ts`
- Modify: `editor/index.html`

**Step 1: Create NewAssetDialog component**

Create `editor/src/components/NewAssetDialog.ts`:

```typescript
/**
 * Dialog for creating new assets with template selection.
 */
import { invoke } from "@tauri-apps/api/core";
import { AssetTypeSelector } from "./AssetTypeSelector";

export class NewAssetDialog {
  private overlay: HTMLDivElement;
  private onTemplateLoaded: (content: string) => void;

  constructor(onTemplateLoaded: (content: string) => void) {
    this.onTemplateLoaded = onTemplateLoaded;
    this.overlay = this.createOverlay();
  }

  private createOverlay(): HTMLDivElement {
    const overlay = document.createElement("div");
    overlay.className = "new-asset-overlay";
    overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.7);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
    `;

    const dialog = document.createElement("div");
    dialog.className = "new-asset-dialog";
    dialog.style.cssText = `
      background: #1e1e1e;
      border: 1px solid #444;
      border-radius: 8px;
      width: 500px;
      max-height: 80vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    `;

    // Header
    const header = document.createElement("div");
    header.style.cssText = `
      padding: 16px;
      border-bottom: 1px solid #333;
      display: flex;
      justify-content: space-between;
      align-items: center;
    `;
    header.innerHTML = `
      <span style="font-size: 14px; font-weight: 500; color: #fff;">Create New Asset</span>
    `;

    const closeBtn = document.createElement("button");
    closeBtn.textContent = "×";
    closeBtn.style.cssText = `
      background: none;
      border: none;
      color: #888;
      font-size: 20px;
      cursor: pointer;
      padding: 0 4px;
    `;
    closeBtn.onclick = () => this.close();
    header.appendChild(closeBtn);
    dialog.appendChild(header);

    // Content
    const content = document.createElement("div");
    content.style.cssText = `
      flex: 1;
      overflow-y: auto;
    `;
    new AssetTypeSelector(content, (templateId) => this.loadTemplate(templateId));
    dialog.appendChild(content);

    overlay.appendChild(dialog);

    // Close on overlay click
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) this.close();
    });

    return overlay;
  }

  private async loadTemplate(templateId: string): Promise<void> {
    try {
      const content = await invoke<string>("plugin:speccade|get_template", {
        id: templateId,
      });
      this.onTemplateLoaded(content);
      this.close();
    } catch (error) {
      console.error("Failed to load template:", error);
    }
  }

  show(): void {
    document.body.appendChild(this.overlay);
  }

  close(): void {
    if (this.overlay.parentNode) {
      this.overlay.parentNode.removeChild(this.overlay);
    }
  }
}
```

**Step 2: Add "New" button to header in index.html**

In `editor/index.html`, update the header div:

```html
<div class="header">
  <span>SpecCade Editor</span>
  <div style="margin-left: auto; display: flex; gap: 8px;">
    <button id="new-asset-btn" style="padding: 4px 12px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;">+ New</button>
  </div>
</div>
```

**Step 3: Wire up button in main.ts**

In `editor/src/main.ts`, add import:

```typescript
import { NewAssetDialog } from "./components/NewAssetDialog";
```

And in `init()`, add after `initEditor()`:

```typescript
// Set up New Asset button
const newAssetBtn = document.getElementById("new-asset-btn");
if (newAssetBtn) {
  newAssetBtn.addEventListener("click", () => {
    const dialog = new NewAssetDialog((content) => {
      if (editor) {
        editor.setContent(content);
        evaluateSource(content);
      }
    });
    dialog.show();
  });
}
```

**Step 4: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 5: Commit**

```bash
git add editor/
git commit -m "feat(editor): add New Asset dialog with template selection"
```

---

### Task 4: Add Keyboard Shortcut for New Asset

**Files:**
- Modify: `editor/src/main.ts`

**Step 1: Add keyboard listener**

In `editor/src/main.ts`, in `init()` after setting up the New Asset button, add:

```typescript
// Keyboard shortcut: Ctrl/Cmd+N for new asset
document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === "n") {
    e.preventDefault();
    const dialog = new NewAssetDialog((content) => {
      if (editor) {
        editor.setContent(content);
        evaluateSource(content);
      }
    });
    dialog.show();
  }
});
```

**Step 2: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add editor/src/main.ts
git commit -m "feat(editor): add Ctrl+N keyboard shortcut for new asset"
```

---

## Phase 2: Project Management

### Task 5: Add Project State Backend

**Files:**
- Create: `crates/speccade-editor/src/commands/project.rs`
- Modify: `crates/speccade-editor/src/commands/mod.rs`
- Modify: `crates/speccade-editor/src/lib.rs`

**Step 1: Create project commands**

Create `crates/speccade-editor/src/commands/project.rs`:

```rust
//! Project management commands for the editor.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File entry for project browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub asset_type: Option<String>,
}

/// Open a folder and list spec files.
#[tauri::command]
pub fn open_folder(path: String) -> Result<Vec<FileEntry>, String> {
    let folder = PathBuf::from(&path);

    if !folder.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(&folder)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();

        // Only include .star files and directories
        let include = is_dir || name.ends_with(".star") || name.ends_with(".json");

        if include {
            let asset_type = if !is_dir {
                detect_asset_type(&path)
            } else {
                None
            };

            entries.push(FileEntry {
                path: path.to_string_lossy().to_string(),
                name,
                is_dir,
                asset_type,
            });
        }
    }

    // Sort: directories first, then files alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(entries)
}

/// Read a file's content.
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Save content to a file.
#[tauri::command]
pub fn save_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to save file: {}", e))
}

/// Detect asset type from file content.
fn detect_asset_type(path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    // Simple heuristic: look for asset_type in the content
    if content.contains("asset_type = \"audio\"") || content.contains("\"asset_type\": \"audio\"") {
        Some("audio".to_string())
    } else if content.contains("asset_type = \"music\"") || content.contains("music_spec(") {
        Some("music".to_string())
    } else if content.contains("asset_type = \"texture\"") || content.contains("\"asset_type\": \"texture\"") {
        Some("texture".to_string())
    } else if content.contains("asset_type = \"static_mesh\"") || content.contains("\"asset_type\": \"static_mesh\"") {
        Some("static_mesh".to_string())
    } else if content.contains("skeletal_mesh_spec(") {
        Some("skeletal_mesh".to_string())
    } else if content.contains("skeletal_animation_spec(") {
        Some("skeletal_animation".to_string())
    } else {
        None
    }
}
```

**Step 2: Register module and commands**

In `crates/speccade-editor/src/commands/mod.rs`, add:

```rust
pub mod project;
```

In `crates/speccade-editor/src/lib.rs`, add:

```rust
pub use commands::project::{open_folder, read_file, save_file};
```

And add to invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    eval_spec,
    validate_spec,
    generate_preview,
    refine_mesh_preview,
    watch_file,
    unwatch_file,
    list_templates,
    get_template,
    open_folder,
    read_file,
    save_file,
])
```

**Step 3: Run tests**

Run: `cd /c/Development/nethercore-project/speccade && cargo test -p tauri-plugin-speccade`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/speccade-editor/
git commit -m "feat(editor): add project management backend (open folder, read/save file)"
```

---

### Task 6: Add File Browser Sidebar

**Files:**
- Create: `editor/src/components/FileBrowser.ts`
- Modify: `editor/src/main.ts`
- Modify: `editor/index.html`

**Step 1: Create FileBrowser component**

Create `editor/src/components/FileBrowser.ts`:

```typescript
/**
 * File browser sidebar for project navigation.
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface FileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  asset_type: string | null;
}

const ASSET_ICONS: Record<string, string> = {
  audio: "🔊",
  music: "🎵",
  texture: "🎨",
  static_mesh: "📦",
  skeletal_mesh: "🧍",
  skeletal_animation: "🏃",
  sprite: "🖼️",
  vfx: "✨",
  folder: "📁",
  file: "📄",
};

export class FileBrowser {
  private container: HTMLElement;
  private currentPath: string | null = null;
  private onFileSelect: (path: string, content: string) => void;

  constructor(container: HTMLElement, onFileSelect: (path: string, content: string) => void) {
    this.container = container;
    this.onFileSelect = onFileSelect;
    this.render();
  }

  private render(): void {
    this.container.innerHTML = "";
    this.container.style.cssText = `
      display: flex;
      flex-direction: column;
      height: 100%;
      background: #1a1a1a;
    `;

    // Header with open folder button
    const header = document.createElement("div");
    header.style.cssText = `
      padding: 8px;
      border-bottom: 1px solid #333;
      display: flex;
      gap: 8px;
    `;

    const openBtn = document.createElement("button");
    openBtn.textContent = "Open Folder";
    openBtn.style.cssText = `
      flex: 1;
      padding: 6px;
      background: #2a2a2a;
      color: #ccc;
      border: 1px solid #444;
      border-radius: 4px;
      cursor: pointer;
      font-size: 11px;
    `;
    openBtn.onclick = () => this.openFolder();
    header.appendChild(openBtn);
    this.container.appendChild(header);

    // File list
    const list = document.createElement("div");
    list.id = "file-list";
    list.style.cssText = `
      flex: 1;
      overflow-y: auto;
      padding: 4px;
    `;
    this.container.appendChild(list);

    if (this.currentPath) {
      this.loadFolder(this.currentPath);
    } else {
      list.innerHTML = `<div style="color: #666; font-size: 11px; padding: 8px; text-align: center;">No folder open</div>`;
    }
  }

  private async openFolder(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        this.currentPath = selected;
        await this.loadFolder(selected);
      }
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  }

  private async loadFolder(path: string): Promise<void> {
    try {
      const entries = await invoke<FileEntry[]>("plugin:speccade|open_folder", { path });
      const list = this.container.querySelector("#file-list");
      if (!list) return;

      list.innerHTML = "";

      // Add parent folder link if not at root
      if (path !== this.currentPath) {
        const parentItem = this.createFileItem({
          path: path.split(/[/\\]/).slice(0, -1).join("/"),
          name: "..",
          is_dir: true,
          asset_type: null,
        });
        list.appendChild(parentItem);
      }

      for (const entry of entries) {
        const item = this.createFileItem(entry);
        list.appendChild(item);
      }
    } catch (error) {
      console.error("Failed to load folder:", error);
    }
  }

  private createFileItem(entry: FileEntry): HTMLElement {
    const item = document.createElement("div");
    item.style.cssText = `
      padding: 4px 8px;
      display: flex;
      align-items: center;
      gap: 6px;
      cursor: pointer;
      border-radius: 4px;
      font-size: 12px;
      color: #ccc;
    `;

    const icon = entry.is_dir
      ? ASSET_ICONS.folder
      : (entry.asset_type ? ASSET_ICONS[entry.asset_type] : ASSET_ICONS.file) || ASSET_ICONS.file;

    item.innerHTML = `
      <span>${icon}</span>
      <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${entry.name}</span>
    `;

    item.addEventListener("mouseenter", () => {
      item.style.background = "#2a2a2a";
    });
    item.addEventListener("mouseleave", () => {
      item.style.background = "transparent";
    });

    item.addEventListener("click", async () => {
      if (entry.is_dir) {
        await this.loadFolder(entry.path);
      } else {
        await this.selectFile(entry.path);
      }
    });

    return item;
  }

  private async selectFile(path: string): Promise<void> {
    try {
      const content = await invoke<string>("plugin:speccade|read_file", { path });
      this.onFileSelect(path, content);
    } catch (error) {
      console.error("Failed to read file:", error);
    }
  }

  dispose(): void {
    this.container.innerHTML = "";
  }
}
```

**Step 2: Update index.html layout**

Update `editor/index.html` to add sidebar:

```html
<div class="main">
  <div class="sidebar" id="sidebar" style="width: 200px; background: #1a1a1a; border-right: 1px solid #333;"></div>
  <div class="editor-pane">
    <div id="editor-container"></div>
  </div>
  <div class="preview-pane">
    <div class="preview-header">Preview</div>
    <div class="preview-content" id="preview-content">
      No preview available
    </div>
  </div>
</div>
```

**Step 3: Initialize FileBrowser in main.ts**

In `editor/src/main.ts`, add import:

```typescript
import { FileBrowser } from "./components/FileBrowser";
```

Add state variable:

```typescript
let fileBrowser: FileBrowser | null = null;
let currentFilePath: string | null = null;
```

In `init()`, add:

```typescript
// Set up file browser sidebar
const sidebar = document.getElementById("sidebar");
if (sidebar) {
  fileBrowser = new FileBrowser(sidebar, (path, content) => {
    currentFilePath = path;
    if (editor) {
      editor.setContent(content);
      evaluateSource(content);
    }
    updateWindowTitle(path);
  });
}
```

Add helper function:

```typescript
function updateWindowTitle(path: string | null): void {
  const filename = path ? path.split(/[/\\]/).pop() : "Untitled";
  document.title = `${filename} - SpecCade Editor`;
}
```

**Step 4: Add dialog plugin to Cargo.toml**

In `editor/src-tauri/Cargo.toml`, add:

```toml
tauri-plugin-dialog = "2"
```

And in `editor/src-tauri/src/main.rs`, add to plugin registration:

```rust
.plugin(tauri_plugin_dialog::init())
```

**Step 5: Verify everything compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npm run build`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add editor/ crates/speccade-editor/
git commit -m "feat(editor): add file browser sidebar with folder navigation"
```

---

### Task 7: Add Save File Functionality

**Files:**
- Modify: `editor/src/main.ts`
- Modify: `editor/index.html`

**Step 1: Add Save button to header**

In `editor/index.html`, update the header buttons:

```html
<div style="margin-left: auto; display: flex; gap: 8px;">
  <button id="new-asset-btn" style="padding: 4px 12px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;">+ New</button>
  <button id="save-btn" style="padding: 4px 12px; background: #2a2a2a; color: #ccc; border: 1px solid #444; border-radius: 4px; cursor: pointer; font-size: 12px;">Save</button>
</div>
```

**Step 2: Add save functionality in main.ts**

In `editor/src/main.ts`, add after the New Asset button setup:

```typescript
// Set up Save button
const saveBtn = document.getElementById("save-btn");
if (saveBtn) {
  saveBtn.addEventListener("click", saveCurrentFile);
}

// Keyboard shortcut: Ctrl/Cmd+S for save
document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === "s") {
    e.preventDefault();
    saveCurrentFile();
  }
});
```

Add the save function:

```typescript
async function saveCurrentFile(): Promise<void> {
  if (!editor) return;

  const content = editor.getContent();

  if (currentFilePath) {
    // Save to existing file
    try {
      await invoke("plugin:speccade|save_file", {
        path: currentFilePath,
        content,
      });
      updateStatus("Saved");
    } catch (error) {
      updateStatus(`Save failed: ${error}`);
    }
  } else {
    // Save as new file
    const { save } = await import("@tauri-apps/plugin-dialog");
    try {
      const path = await save({
        filters: [{ name: "Starlark", extensions: ["star"] }],
      });

      if (path) {
        await invoke("plugin:speccade|save_file", {
          path,
          content,
        });
        currentFilePath = path;
        updateWindowTitle(path);
        updateStatus("Saved");
      }
    } catch (error) {
      updateStatus(`Save failed: ${error}`);
    }
  }
}
```

**Step 3: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add editor/
git commit -m "feat(editor): add save file functionality with Ctrl+S shortcut"
```

---

### Task 8: Add Recent Files Menu

**Files:**
- Create: `editor/src/lib/recent-files.ts`
- Modify: `editor/src/main.ts`

**Step 1: Create recent files manager**

Create `editor/src/lib/recent-files.ts`:

```typescript
/**
 * Recent files manager using localStorage.
 */

const STORAGE_KEY = "speccade-recent-files";
const MAX_RECENT = 10;

export interface RecentFile {
  path: string;
  name: string;
  timestamp: number;
}

export function getRecentFiles(): RecentFile[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

export function addRecentFile(path: string): void {
  const name = path.split(/[/\\]/).pop() || path;
  const files = getRecentFiles().filter((f) => f.path !== path);

  files.unshift({
    path,
    name,
    timestamp: Date.now(),
  });

  if (files.length > MAX_RECENT) {
    files.pop();
  }

  localStorage.setItem(STORAGE_KEY, JSON.stringify(files));
}

export function clearRecentFiles(): void {
  localStorage.removeItem(STORAGE_KEY);
}
```

**Step 2: Update file selection to track recent files**

In `editor/src/main.ts`, add import:

```typescript
import { addRecentFile, getRecentFiles } from "./lib/recent-files";
```

Update the file browser callback in `init()`:

```typescript
fileBrowser = new FileBrowser(sidebar, (path, content) => {
  currentFilePath = path;
  addRecentFile(path);  // Add this line
  if (editor) {
    editor.setContent(content);
    evaluateSource(content);
  }
  updateWindowTitle(path);
});
```

**Step 3: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add editor/src/lib/recent-files.ts editor/src/main.ts
git commit -m "feat(editor): add recent files tracking with localStorage"
```

---

## Phase 3: Export & Generation

### Task 9: Add Generate Panel Backend

**Files:**
- Create: `crates/speccade-editor/src/commands/generate.rs` (rename existing)
- Modify: `crates/speccade-editor/src/lib.rs`

**Step 1: Add full generation command**

Add to `crates/speccade-editor/src/commands/generate.rs`:

```rust
/// Output from full asset generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFullOutput {
    pub success: bool,
    pub outputs: Vec<GeneratedFile>,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

/// A generated output file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub size_bytes: u64,
    pub format: String,
}

/// Generate full assets from a spec file.
#[tauri::command]
pub fn generate_full(
    source: String,
    filename: String,
    output_dir: String,
) -> GenerateFullOutput {
    use std::time::Instant;
    let start = Instant::now();

    // Configure compiler
    let config = CompilerConfig::default();

    // Compile the spec
    let spec = match compiler::compile(&filename, &source, &config) {
        Ok(result) => result.spec,
        Err(e) => {
            return GenerateFullOutput {
                success: false,
                outputs: vec![],
                error: Some(format!("Compilation error: {}", e)),
                elapsed_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    // Run full generation using speccade-cli dispatch
    match speccade_cli::dispatch::generate_spec(&spec, std::path::Path::new(&output_dir)) {
        Ok(results) => {
            let outputs: Vec<GeneratedFile> = results
                .iter()
                .map(|r| GeneratedFile {
                    path: r.path.to_string_lossy().to_string(),
                    size_bytes: r.size_bytes,
                    format: r.format.clone(),
                })
                .collect();

            GenerateFullOutput {
                success: true,
                outputs,
                error: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(e) => GenerateFullOutput {
            success: false,
            outputs: vec![],
            error: Some(format!("Generation error: {}", e)),
            elapsed_ms: start.elapsed().as_millis() as u64,
        },
    }
}
```

**Step 2: Register the command**

In `crates/speccade-editor/src/lib.rs`, add:

```rust
pub use commands::generate::generate_full;
```

And add to invoke_handler:

```rust
generate_full,
```

**Step 3: Run tests**

Run: `cd /c/Development/nethercore-project/speccade && cargo test -p tauri-plugin-speccade`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/speccade-editor/
git commit -m "feat(editor): add full asset generation backend command"
```

---

### Task 10: Add Generate Panel UI

**Files:**
- Create: `editor/src/components/GeneratePanel.ts`
- Modify: `editor/src/main.ts`
- Modify: `editor/index.html`

**Step 1: Create GeneratePanel component**

Create `editor/src/components/GeneratePanel.ts`:

```typescript
/**
 * Panel for generating full-quality assets.
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface GeneratedFile {
  path: string;
  size_bytes: number;
  format: string;
}

interface GenerateFullOutput {
  success: boolean;
  outputs: GeneratedFile[];
  error: string | null;
  elapsed_ms: number;
}

export class GeneratePanel {
  private container: HTMLElement;
  private getSource: () => string;
  private outputDir: string | null = null;

  constructor(container: HTMLElement, getSource: () => string) {
    this.container = container;
    this.getSource = getSource;
    this.render();
  }

  private render(): void {
    this.container.innerHTML = "";
    this.container.style.cssText = `
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 8px;
    `;

    // Output directory selector
    const dirRow = document.createElement("div");
    dirRow.style.cssText = `
      display: flex;
      gap: 8px;
      align-items: center;
    `;

    const dirLabel = document.createElement("span");
    dirLabel.textContent = "Output:";
    dirLabel.style.cssText = "font-size: 11px; color: #888;";
    dirRow.appendChild(dirLabel);

    const dirPath = document.createElement("span");
    dirPath.id = "output-dir-path";
    dirPath.textContent = this.outputDir || "(not set)";
    dirPath.style.cssText = `
      flex: 1;
      font-size: 11px;
      color: #ccc;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    `;
    dirRow.appendChild(dirPath);

    const browseBtn = document.createElement("button");
    browseBtn.textContent = "Browse";
    browseBtn.style.cssText = `
      padding: 4px 8px;
      background: #2a2a2a;
      color: #ccc;
      border: 1px solid #444;
      border-radius: 4px;
      cursor: pointer;
      font-size: 11px;
    `;
    browseBtn.onclick = () => this.selectOutputDir();
    dirRow.appendChild(browseBtn);

    this.container.appendChild(dirRow);

    // Generate button
    const generateBtn = document.createElement("button");
    generateBtn.id = "generate-btn";
    generateBtn.textContent = "Generate Assets";
    generateBtn.style.cssText = `
      padding: 8px 16px;
      background: #007acc;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
      font-weight: 500;
    `;
    generateBtn.onclick = () => this.generate();
    this.container.appendChild(generateBtn);

    // Results area
    const results = document.createElement("div");
    results.id = "generate-results";
    results.style.cssText = `
      flex: 1;
      overflow-y: auto;
      font-size: 11px;
      color: #888;
    `;
    this.container.appendChild(results);
  }

  private async selectOutputDir(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        this.outputDir = selected;
        const pathEl = this.container.querySelector("#output-dir-path");
        if (pathEl) {
          pathEl.textContent = selected;
        }
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    }
  }

  private async generate(): Promise<void> {
    if (!this.outputDir) {
      await this.selectOutputDir();
      if (!this.outputDir) return;
    }

    const source = this.getSource();
    const results = this.container.querySelector("#generate-results");
    const btn = this.container.querySelector("#generate-btn") as HTMLButtonElement;

    if (!results || !btn) return;

    btn.disabled = true;
    btn.textContent = "Generating...";
    results.innerHTML = "";

    try {
      const output = await invoke<GenerateFullOutput>("plugin:speccade|generate_full", {
        source,
        filename: "editor.star",
        outputDir: this.outputDir,
      });

      if (output.success) {
        results.innerHTML = `
          <div style="color: #4caf50; margin-bottom: 8px;">
            Generated ${output.outputs.length} file(s) in ${output.elapsed_ms}ms
          </div>
          ${output.outputs.map((f) => `
            <div style="padding: 4px 0; border-bottom: 1px solid #333;">
              <div style="color: #ccc;">${f.path.split(/[/\\]/).pop()}</div>
              <div style="color: #666; font-size: 10px;">${this.formatBytes(f.size_bytes)} • ${f.format}</div>
            </div>
          `).join("")}
        `;
      } else {
        results.innerHTML = `<div style="color: #f44336;">${output.error}</div>`;
      }
    } catch (error) {
      results.innerHTML = `<div style="color: #f44336;">Error: ${error}</div>`;
    } finally {
      btn.disabled = false;
      btn.textContent = "Generate Assets";
    }
  }

  private formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  dispose(): void {
    this.container.innerHTML = "";
  }
}
```

**Step 2: Add generate panel to layout**

In `editor/index.html`, update the preview-pane to include tabs:

```html
<div class="preview-pane">
  <div class="preview-header" style="display: flex; gap: 0;">
    <button id="preview-tab" class="tab-btn active" style="flex: 1; padding: 8px; background: #252525; border: none; color: #fff; cursor: pointer; font-size: 12px;">Preview</button>
    <button id="generate-tab" class="tab-btn" style="flex: 1; padding: 8px; background: #1a1a1a; border: none; color: #888; cursor: pointer; font-size: 12px;">Generate</button>
  </div>
  <div class="preview-content" id="preview-content">
    No preview available
  </div>
  <div class="generate-content" id="generate-content" style="display: none; flex: 1;">
  </div>
</div>
```

**Step 3: Wire up tabs in main.ts**

In `editor/src/main.ts`, add import:

```typescript
import { GeneratePanel } from "./components/GeneratePanel";
```

Add state:

```typescript
let generatePanel: GeneratePanel | null = null;
```

In `init()`, add:

```typescript
// Set up generate panel
const generateContent = document.getElementById("generate-content");
if (generateContent && editor) {
  generatePanel = new GeneratePanel(generateContent, () => editor!.getContent());
}

// Tab switching
const previewTab = document.getElementById("preview-tab");
const generateTab = document.getElementById("generate-tab");
const previewContent = document.getElementById("preview-content");

if (previewTab && generateTab && previewContent && generateContent) {
  previewTab.addEventListener("click", () => {
    previewTab.style.background = "#252525";
    previewTab.style.color = "#fff";
    generateTab.style.background = "#1a1a1a";
    generateTab.style.color = "#888";
    previewContent.style.display = "flex";
    generateContent.style.display = "none";
  });

  generateTab.addEventListener("click", () => {
    generateTab.style.background = "#252525";
    generateTab.style.color = "#fff";
    previewTab.style.background = "#1a1a1a";
    previewTab.style.color = "#888";
    generateContent.style.display = "flex";
    previewContent.style.display = "none";
  });
}
```

**Step 4: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 5: Commit**

```bash
git add editor/
git commit -m "feat(editor): add generate panel with output directory and progress"
```

---

### Task 11: Add Batch Generation

**Files:**
- Create: `crates/speccade-editor/src/commands/batch.rs`
- Modify: `crates/speccade-editor/src/commands/mod.rs`
- Modify: `crates/speccade-editor/src/lib.rs`

**Step 1: Create batch generation command**

Create `crates/speccade-editor/src/commands/batch.rs`:

```rust
//! Batch generation commands for generating multiple specs at once.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::generate::{GenerateFullOutput, GeneratedFile};

/// Output from batch generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGenerateOutput {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
    pub elapsed_ms: u64,
}

/// Result for a single batch item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub path: String,
    pub success: bool,
    pub outputs: Vec<GeneratedFile>,
    pub error: Option<String>,
}

/// Generate multiple spec files in batch.
#[tauri::command]
pub fn batch_generate(
    paths: Vec<String>,
    output_dir: String,
) -> BatchGenerateOutput {
    use std::time::Instant;
    use speccade_cli::compiler::{self, CompilerConfig};

    let start = Instant::now();
    let config = CompilerConfig::default();
    let out_path = Path::new(&output_dir);

    let mut results = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;

    for path in &paths {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                failed += 1;
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: false,
                    outputs: vec![],
                    error: Some(format!("Failed to read file: {}", e)),
                });
                continue;
            }
        };

        let filename = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.star".to_string());

        // Compile
        let spec = match compiler::compile(&filename, &source, &config) {
            Ok(result) => result.spec,
            Err(e) => {
                failed += 1;
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: false,
                    outputs: vec![],
                    error: Some(format!("Compilation error: {}", e)),
                });
                continue;
            }
        };

        // Generate
        match speccade_cli::dispatch::generate_spec(&spec, out_path) {
            Ok(gen_results) => {
                succeeded += 1;
                let outputs: Vec<GeneratedFile> = gen_results
                    .iter()
                    .map(|r| GeneratedFile {
                        path: r.path.to_string_lossy().to_string(),
                        size_bytes: r.size_bytes,
                        format: r.format.clone(),
                    })
                    .collect();
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: true,
                    outputs,
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: false,
                    outputs: vec![],
                    error: Some(format!("Generation error: {}", e)),
                });
            }
        }
    }

    BatchGenerateOutput {
        total: paths.len(),
        succeeded,
        failed,
        results,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}
```

**Step 2: Register module and command**

In `crates/speccade-editor/src/commands/mod.rs`, add:

```rust
pub mod batch;
```

In `crates/speccade-editor/src/lib.rs`, add:

```rust
pub use commands::batch::batch_generate;
```

And add to invoke_handler:

```rust
batch_generate,
```

**Step 3: Run tests**

Run: `cd /c/Development/nethercore-project/speccade && cargo test -p tauri-plugin-speccade`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/speccade-editor/
git commit -m "feat(editor): add batch generation backend for multiple specs"
```

---

### Task 12: Add Pack Manifest Generation

**Files:**
- Create: `crates/speccade-editor/src/commands/pack.rs`
- Modify: `crates/speccade-editor/src/commands/mod.rs`
- Modify: `crates/speccade-editor/src/lib.rs`

**Step 1: Create pack manifest command**

Create `crates/speccade-editor/src/commands/pack.rs`:

```rust
//! Pack manifest generation for bundling assets.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A pack manifest entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackEntry {
    pub asset_id: String,
    pub asset_type: String,
    pub path: String,
    pub size_bytes: u64,
    pub spec_hash: String,
}

/// Pack manifest output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub version: u32,
    pub name: String,
    pub entries: Vec<PackEntry>,
    pub total_size_bytes: u64,
}

/// Generate a pack manifest from a directory of generated assets.
#[tauri::command]
pub fn generate_pack_manifest(
    name: String,
    output_dir: String,
) -> Result<PackManifest, String> {
    let path = Path::new(&output_dir);

    if !path.is_dir() {
        return Err(format!("Not a directory: {}", output_dir));
    }

    let mut entries = Vec::new();
    let mut total_size = 0u64;

    // Walk the directory and collect asset files
    fn walk_dir(dir: &Path, entries: &mut Vec<PackEntry>, total_size: &mut u64) -> Result<(), String> {
        let read_dir = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_dir() {
                walk_dir(&path, entries, total_size)?;
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let asset_type = match ext {
                    "wav" | "ogg" => "audio",
                    "xm" | "it" => "music",
                    "png" | "jpg" => "texture",
                    "glb" | "gltf" => "mesh",
                    _ => continue,
                };

                let metadata = std::fs::metadata(&path)
                    .map_err(|e| format!("Failed to get metadata: {}", e))?;

                let size = metadata.len();
                *total_size += size;

                let name = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Simple hash of path for now
                let hash = format!("{:x}", blake3::hash(path.to_string_lossy().as_bytes()));

                entries.push(PackEntry {
                    asset_id: name,
                    asset_type: asset_type.to_string(),
                    path: path.to_string_lossy().to_string(),
                    size_bytes: size,
                    spec_hash: hash[..16].to_string(),
                });
            }
        }

        Ok(())
    }

    walk_dir(path, &mut entries, &mut total_size)?;

    Ok(PackManifest {
        version: 1,
        name,
        entries,
        total_size_bytes: total_size,
    })
}

/// Write pack manifest to JSON file.
#[tauri::command]
pub fn write_pack_manifest(
    manifest: PackManifest,
    output_path: String,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    std::fs::write(&output_path, json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}
```

**Step 2: Register module and commands**

In `crates/speccade-editor/src/commands/mod.rs`, add:

```rust
pub mod pack;
```

In `crates/speccade-editor/src/lib.rs`, add:

```rust
pub use commands::pack::{generate_pack_manifest, write_pack_manifest};
```

And add to invoke_handler:

```rust
generate_pack_manifest,
write_pack_manifest,
```

**Step 3: Run tests**

Run: `cd /c/Development/nethercore-project/speccade && cargo test -p tauri-plugin-speccade`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/speccade-editor/
git commit -m "feat(editor): add pack manifest generation for asset bundling"
```

---

## Phase 4: Discoverability

### Task 13: Add Stdlib Palette Panel

**Files:**
- Create: `editor/src/components/StdlibPalette.ts`

**Step 1: Create StdlibPalette component**

Create `editor/src/components/StdlibPalette.ts`:

```typescript
/**
 * Stdlib function palette for quick insertion.
 */

interface FunctionInfo {
  name: string;
  signature: string;
  description: string;
  snippet: string;
}

interface CategoryInfo {
  name: string;
  icon: string;
  functions: FunctionInfo[];
}

const STDLIB_CATEGORIES: CategoryInfo[] = [
  {
    name: "Audio Synthesis",
    icon: "🔊",
    functions: [
      {
        name: "oscillator",
        signature: "oscillator(freq, waveform, [end_freq, sweep])",
        description: "Basic oscillator synthesis",
        snippet: 'oscillator(440, "sine")',
      },
      {
        name: "fm_synth",
        signature: "fm_synth(carrier, modulator, mod_index)",
        description: "FM synthesis with carrier and modulator",
        snippet: 'fm_synth(440, 2.0, 5.0)',
      },
      {
        name: "envelope",
        signature: "envelope(attack, decay, sustain, release)",
        description: "ADSR envelope",
        snippet: "envelope(0.01, 0.1, 0.5, 0.2)",
      },
      {
        name: "audio_layer",
        signature: "audio_layer(synthesis, envelope, [filter])",
        description: "Complete audio synthesis layer",
        snippet: 'audio_layer(\n    oscillator(440, "sine"),\n    envelope(0.01, 0.1, 0.5, 0.2)\n)',
      },
    ],
  },
  {
    name: "Audio Filters",
    icon: "🎛️",
    functions: [
      {
        name: "lowpass",
        signature: "lowpass(cutoff, resonance, [end_cutoff])",
        description: "Lowpass filter",
        snippet: "lowpass(2000, 0.707)",
      },
      {
        name: "highpass",
        signature: "highpass(cutoff, resonance)",
        description: "Highpass filter",
        snippet: "highpass(500, 0.707)",
      },
      {
        name: "reverb",
        signature: "reverb([room_size, damping, wet])",
        description: "Reverb effect",
        snippet: "reverb(0.5, 0.5, 0.3)",
      },
    ],
  },
  {
    name: "Texture Nodes",
    icon: "🎨",
    functions: [
      {
        name: "noise_node",
        signature: 'noise_node(id, type, scale, [octaves])',
        description: "Noise texture node",
        snippet: 'noise_node("noise", "perlin", 0.05, 4)',
      },
      {
        name: "color_ramp_node",
        signature: "color_ramp_node(id, input, colors)",
        description: "Map grayscale to colors",
        snippet: 'color_ramp_node("colored", "noise", ["#000000", "#ffffff"])',
      },
      {
        name: "texture_graph",
        signature: "texture_graph(size, nodes)",
        description: "Complete texture graph",
        snippet: "texture_graph(\n    [256, 256],\n    []\n)",
      },
    ],
  },
  {
    name: "Mesh",
    icon: "📦",
    functions: [
      {
        name: "mesh_recipe",
        signature: "mesh_recipe(primitive, dimensions, [modifiers])",
        description: "Complete mesh recipe",
        snippet: 'mesh_recipe("cube", [1.0, 1.0, 1.0], [])',
      },
      {
        name: "bevel_modifier",
        signature: "bevel_modifier(width, segments)",
        description: "Bevel modifier",
        snippet: "bevel_modifier(0.05, 2)",
      },
      {
        name: "subdivision_modifier",
        signature: "subdivision_modifier(levels)",
        description: "Subdivision surface modifier",
        snippet: "subdivision_modifier(1)",
      },
    ],
  },
  {
    name: "Core",
    icon: "⚙️",
    functions: [
      {
        name: "spec",
        signature: "spec(asset_id, asset_type, seed, outputs, recipe)",
        description: "Create a complete spec",
        snippet: 'spec(\n    asset_id = "my-asset",\n    asset_type = "audio",\n    seed = 42,\n    outputs = [],\n    recipe = {}\n)',
      },
      {
        name: "output",
        signature: 'output(path, format)',
        description: "Create an output specification",
        snippet: 'output("path/to/file.wav", "wav")',
      },
    ],
  },
];

export class StdlibPalette {
  private container: HTMLElement;
  private onInsert: (snippet: string) => void;

  constructor(container: HTMLElement, onInsert: (snippet: string) => void) {
    this.container = container;
    this.onInsert = onInsert;
    this.render();
  }

  private render(): void {
    this.container.innerHTML = "";
    this.container.style.cssText = `
      display: flex;
      flex-direction: column;
      height: 100%;
      overflow-y: auto;
      padding: 8px;
      gap: 8px;
    `;

    for (const category of STDLIB_CATEGORIES) {
      const section = this.createCategory(category);
      this.container.appendChild(section);
    }
  }

  private createCategory(category: CategoryInfo): HTMLElement {
    const section = document.createElement("div");
    section.style.cssText = `
      display: flex;
      flex-direction: column;
      gap: 4px;
    `;

    const header = document.createElement("div");
    header.style.cssText = `
      font-size: 11px;
      font-weight: 500;
      color: #888;
      padding: 4px 0;
      display: flex;
      align-items: center;
      gap: 4px;
    `;
    header.innerHTML = `<span>${category.icon}</span> ${category.name}`;
    section.appendChild(header);

    for (const fn of category.functions) {
      const item = this.createFunctionItem(fn);
      section.appendChild(item);
    }

    return section;
  }

  private createFunctionItem(fn: FunctionInfo): HTMLElement {
    const item = document.createElement("div");
    item.style.cssText = `
      padding: 4px 8px;
      background: #2a2a2a;
      border-radius: 4px;
      cursor: pointer;
      font-size: 11px;
    `;
    item.innerHTML = `
      <div style="color: #9cdcfe; font-family: monospace;">${fn.name}()</div>
      <div style="color: #666; font-size: 10px; margin-top: 2px;">${fn.description}</div>
    `;

    item.addEventListener("mouseenter", () => {
      item.style.background = "#333";
    });
    item.addEventListener("mouseleave", () => {
      item.style.background = "#2a2a2a";
    });
    item.addEventListener("click", () => {
      this.onInsert(fn.snippet);
    });

    return item;
  }

  dispose(): void {
    this.container.innerHTML = "";
  }
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add editor/src/components/StdlibPalette.ts
git commit -m "feat(editor): add stdlib palette for function discovery and insertion"
```

---

### Task 14: Add Snippets Panel to Sidebar

**Files:**
- Modify: `editor/src/main.ts`
- Modify: `editor/index.html`

**Step 1: Update sidebar layout**

In `editor/index.html`, update the sidebar div:

```html
<div class="sidebar" id="sidebar" style="width: 220px; background: #1a1a1a; border-right: 1px solid #333; display: flex; flex-direction: column;">
  <div class="sidebar-tabs" style="display: flex; border-bottom: 1px solid #333;">
    <button id="files-tab" class="sidebar-tab active" style="flex: 1; padding: 8px 0; background: #252525; border: none; color: #fff; cursor: pointer; font-size: 11px;">Files</button>
    <button id="snippets-tab" class="sidebar-tab" style="flex: 1; padding: 8px 0; background: #1a1a1a; border: none; color: #888; cursor: pointer; font-size: 11px;">Snippets</button>
  </div>
  <div id="files-panel" style="flex: 1; overflow: hidden;"></div>
  <div id="snippets-panel" style="flex: 1; overflow: hidden; display: none;"></div>
</div>
```

**Step 2: Initialize snippets panel in main.ts**

In `editor/src/main.ts`, add import:

```typescript
import { StdlibPalette } from "./components/StdlibPalette";
```

Add state:

```typescript
let stdlibPalette: StdlibPalette | null = null;
```

In `init()`, update sidebar initialization:

```typescript
// Set up file browser sidebar
const filesPanel = document.getElementById("files-panel");
if (filesPanel) {
  fileBrowser = new FileBrowser(filesPanel, (path, content) => {
    currentFilePath = path;
    addRecentFile(path);
    if (editor) {
      editor.setContent(content);
      evaluateSource(content);
    }
    updateWindowTitle(path);
  });
}

// Set up snippets panel
const snippetsPanel = document.getElementById("snippets-panel");
if (snippetsPanel && editor) {
  stdlibPalette = new StdlibPalette(snippetsPanel, (snippet) => {
    if (editor) {
      const monacoEditor = editor.getMonacoEditor();
      const selection = monacoEditor.getSelection();
      if (selection) {
        monacoEditor.executeEdits("snippet", [{
          range: selection,
          text: snippet,
        }]);
        monacoEditor.focus();
      }
    }
  });
}

// Sidebar tab switching
const filesTab = document.getElementById("files-tab");
const snippetsTab = document.getElementById("snippets-tab");

if (filesTab && snippetsTab && filesPanel && snippetsPanel) {
  filesTab.addEventListener("click", () => {
    filesTab.style.background = "#252525";
    filesTab.style.color = "#fff";
    snippetsTab.style.background = "#1a1a1a";
    snippetsTab.style.color = "#888";
    filesPanel.style.display = "block";
    snippetsPanel.style.display = "none";
  });

  snippetsTab.addEventListener("click", () => {
    snippetsTab.style.background = "#252525";
    snippetsTab.style.color = "#fff";
    filesTab.style.background = "#1a1a1a";
    filesTab.style.color = "#888";
    snippetsPanel.style.display = "block";
    filesPanel.style.display = "none";
  });
}
```

**Step 3: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add editor/
git commit -m "feat(editor): add snippets panel to sidebar with stdlib functions"
```

---

### Task 15: Add Monaco Autocomplete for Stdlib

**Files:**
- Create: `editor/src/lib/starlark-completions.ts`
- Modify: `editor/src/components/Editor.ts`

**Step 1: Create completion provider**

Create `editor/src/lib/starlark-completions.ts`:

```typescript
/**
 * Monaco completion provider for SpecCade stdlib functions.
 */
import * as monaco from "monaco-editor";

interface CompletionItem {
  label: string;
  kind: monaco.languages.CompletionItemKind;
  insertText: string;
  insertTextRules: monaco.languages.CompletionItemInsertTextRule;
  documentation: string;
  detail: string;
}

const STDLIB_COMPLETIONS: CompletionItem[] = [
  // Core
  {
    label: "spec",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'spec(\n\tasset_id = "${1:my-asset}",\n\tasset_type = "${2:audio}",\n\tseed = ${3:42},\n\toutputs = [${4}],\n\trecipe = {${5}}\n)',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Create a complete spec dictionary",
    detail: "(id, type, seed, outputs, recipe) -> dict",
  },
  {
    label: "output",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'output("${1:path/to/file}", "${2:wav}")',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Create an output specification",
    detail: "(path, format) -> dict",
  },
  // Audio
  {
    label: "oscillator",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'oscillator(${1:440}, "${2:sine}")',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Basic oscillator synthesis",
    detail: "(freq, waveform, [end_freq, sweep]) -> dict",
  },
  {
    label: "envelope",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "envelope(${1:0.01}, ${2:0.1}, ${3:0.5}, ${4:0.2})",
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "ADSR envelope",
    detail: "(attack, decay, sustain, release) -> dict",
  },
  {
    label: "audio_layer",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "audio_layer(\n\t${1:synthesis},\n\t${2:envelope}\n)",
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete audio synthesis layer",
    detail: "(synthesis, envelope, [filter]) -> dict",
  },
  {
    label: "lowpass",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "lowpass(${1:2000}, ${2:0.707})",
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Lowpass filter",
    detail: "(cutoff, resonance, [end_cutoff]) -> dict",
  },
  {
    label: "reverb",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "reverb(${1:0.5}, ${2:0.5}, ${3:0.3})",
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Reverb effect",
    detail: "([room_size, damping, wet]) -> dict",
  },
  // Texture
  {
    label: "noise_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'noise_node("${1:noise}", "${2:perlin}", ${3:0.05}, ${4:4})',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Noise texture node",
    detail: '(id, type, scale, [octaves]) -> dict',
  },
  {
    label: "color_ramp_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'color_ramp_node("${1:colored}", "${2:input}", ["${3:#000000}", "${4:#ffffff}"])',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Map grayscale to colors",
    detail: "(id, input, colors) -> dict",
  },
  {
    label: "texture_graph",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "texture_graph(\n\t[${1:256}, ${2:256}],\n\t[${3}]\n)",
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete texture graph",
    detail: "(size, nodes) -> dict",
  },
  // Mesh
  {
    label: "mesh_recipe",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'mesh_recipe("${1:cube}", [${2:1.0}, ${3:1.0}, ${4:1.0}], [${5}])',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete mesh recipe",
    detail: "(primitive, dimensions, [modifiers]) -> dict",
  },
  {
    label: "bevel_modifier",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "bevel_modifier(${1:0.05}, ${2:2})",
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Bevel modifier",
    detail: "(width, segments) -> dict",
  },
];

export function registerStarlarkCompletions(): void {
  monaco.languages.registerCompletionItemProvider("starlark", {
    provideCompletionItems: (model, position) => {
      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      const suggestions = STDLIB_COMPLETIONS.map((item) => ({
        ...item,
        range,
      }));

      return { suggestions };
    },
  });
}
```

**Step 2: Register completions in Editor.ts**

In `editor/src/components/Editor.ts`, add import at top:

```typescript
import { registerStarlarkCompletions } from "../lib/starlark-completions";
```

In `registerStarlarkLanguage()`, add at the end:

```typescript
// Register completion provider
registerStarlarkCompletions();
```

**Step 3: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add editor/src/lib/starlark-completions.ts editor/src/components/Editor.ts
git commit -m "feat(editor): add Monaco autocomplete for stdlib functions"
```

---

### Task 16: Add Help Panel with Quick Reference

**Files:**
- Create: `editor/src/components/HelpPanel.ts`
- Modify: `editor/index.html`
- Modify: `editor/src/main.ts`

**Step 1: Create HelpPanel component**

Create `editor/src/components/HelpPanel.ts`:

```typescript
/**
 * Help panel with quick reference and keyboard shortcuts.
 */

export class HelpPanel {
  private overlay: HTMLDivElement;

  constructor() {
    this.overlay = this.createOverlay();
  }

  private createOverlay(): HTMLDivElement {
    const overlay = document.createElement("div");
    overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.7);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
    `;

    const dialog = document.createElement("div");
    dialog.style.cssText = `
      background: #1e1e1e;
      border: 1px solid #444;
      border-radius: 8px;
      width: 600px;
      max-height: 80vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    `;

    dialog.innerHTML = `
      <div style="padding: 16px; border-bottom: 1px solid #333; display: flex; justify-content: space-between; align-items: center;">
        <span style="font-size: 14px; font-weight: 500; color: #fff;">Help & Keyboard Shortcuts</span>
        <button id="help-close" style="background: none; border: none; color: #888; font-size: 20px; cursor: pointer;">×</button>
      </div>
      <div style="flex: 1; overflow-y: auto; padding: 16px;">
        <h3 style="color: #fff; font-size: 12px; margin: 0 0 12px 0;">Keyboard Shortcuts</h3>
        <table style="width: 100%; font-size: 12px; color: #ccc; border-collapse: collapse;">
          <tr>
            <td style="padding: 4px 8px; color: #888;">Ctrl+N</td>
            <td style="padding: 4px 8px;">Create new asset</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">Ctrl+S</td>
            <td style="padding: 4px 8px;">Save current file</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">Ctrl+Space</td>
            <td style="padding: 4px 8px;">Trigger autocomplete</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">F1</td>
            <td style="padding: 4px 8px;">Show this help</td>
          </tr>
        </table>

        <h3 style="color: #fff; font-size: 12px; margin: 24px 0 12px 0;">Asset Types</h3>
        <table style="width: 100%; font-size: 12px; color: #ccc; border-collapse: collapse;">
          <tr>
            <td style="padding: 4px 8px; color: #888;">audio</td>
            <td style="padding: 4px 8px;">Procedural sound effects (lasers, explosions, UI)</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">music</td>
            <td style="padding: 4px 8px;">Tracker-style music (XM/IT format)</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">texture</td>
            <td style="padding: 4px 8px;">Procedural textures and PBR materials</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">static_mesh</td>
            <td style="padding: 4px 8px;">3D meshes from primitives with modifiers</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">skeletal_mesh</td>
            <td style="padding: 4px 8px;">Rigged character meshes</td>
          </tr>
          <tr>
            <td style="padding: 4px 8px; color: #888;">skeletal_animation</td>
            <td style="padding: 4px 8px;">Skeletal animations with keyframes</td>
          </tr>
        </table>

        <h3 style="color: #fff; font-size: 12px; margin: 24px 0 12px 0;">Quick Links</h3>
        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
          <a href="https://github.com/speccade/speccade/blob/main/docs/stdlib-reference.md" target="_blank" style="color: #4a9eff; font-size: 12px; text-decoration: none;">Stdlib Reference</a>
          <a href="https://github.com/speccade/speccade/blob/main/docs/starlark-authoring.md" target="_blank" style="color: #4a9eff; font-size: 12px; text-decoration: none;">Authoring Guide</a>
        </div>
      </div>
    `;

    overlay.appendChild(dialog);

    // Close on overlay click or button
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) this.close();
    });
    dialog.querySelector("#help-close")?.addEventListener("click", () => this.close());

    return overlay;
  }

  show(): void {
    document.body.appendChild(this.overlay);
  }

  close(): void {
    if (this.overlay.parentNode) {
      this.overlay.parentNode.removeChild(this.overlay);
    }
  }
}
```

**Step 2: Add help button and F1 shortcut**

In `editor/index.html`, add help button to header:

```html
<div style="margin-left: auto; display: flex; gap: 8px;">
  <button id="new-asset-btn" style="padding: 4px 12px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;">+ New</button>
  <button id="save-btn" style="padding: 4px 12px; background: #2a2a2a; color: #ccc; border: 1px solid #444; border-radius: 4px; cursor: pointer; font-size: 12px;">Save</button>
  <button id="help-btn" style="padding: 4px 12px; background: #2a2a2a; color: #ccc; border: 1px solid #444; border-radius: 4px; cursor: pointer; font-size: 12px;">?</button>
</div>
```

In `editor/src/main.ts`, add import:

```typescript
import { HelpPanel } from "./components/HelpPanel";
```

In `init()`, add:

```typescript
// Set up Help button
const helpBtn = document.getElementById("help-btn");
if (helpBtn) {
  helpBtn.addEventListener("click", () => {
    new HelpPanel().show();
  });
}

// F1 for help
document.addEventListener("keydown", (e) => {
  if (e.key === "F1") {
    e.preventDefault();
    new HelpPanel().show();
  }
});
```

**Step 3: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add editor/
git commit -m "feat(editor): add help panel with keyboard shortcuts and quick reference"
```

---

## Final Integration Test

### Task 17: Build and Test All Features

**Step 1: Build the editor**

Run: `cd /c/Development/nethercore-project/speccade && cargo build -p tauri-plugin-speccade`
Expected: Build succeeds

Run: `cd /c/Development/nethercore-project/speccade/editor && npm run build`
Expected: Build succeeds

**Step 2: Run Tauri dev mode**

Run: `cd /c/Development/nethercore-project/speccade/editor && npm run tauri dev`
Expected: Editor window opens

**Step 3: Manual testing checklist**

- [ ] Click "+ New" button, verify asset type selector appears
- [ ] Select "Sound Effect", verify template loads in editor
- [ ] Try autocomplete with Ctrl+Space, verify stdlib suggestions
- [ ] Click "Open Folder" in sidebar, select a folder with .star files
- [ ] Click on a file in the browser, verify it loads
- [ ] Edit the file, press Ctrl+S, verify it saves
- [ ] Switch to "Snippets" tab, click a function, verify it inserts
- [ ] Switch to "Generate" tab, select output folder, click Generate
- [ ] Press F1, verify help panel appears
- [ ] Press Ctrl+N, verify new asset dialog appears

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(editor): complete feature expansion

- Asset type wizards: 10 templates covering all asset types
- Project management: file browser, save/load, recent files
- Export capabilities: full generation, batch, pack manifests
- Discoverability: stdlib palette, autocomplete, help panel

This transforms the editor from a single-file text box into a
complete asset authoring workspace."
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1: Asset Types | 1-4 | Asset type selector, templates, new asset dialog |
| 2: Project Mgmt | 5-8 | File browser, save/load, recent files |
| 3: Export | 9-12 | Full generation, batch, pack manifests |
| 4: Discoverability | 13-16 | Stdlib palette, autocomplete, help panel |
| Final | 17 | Integration testing |

Total: 17 tasks, ~20 files created/modified
