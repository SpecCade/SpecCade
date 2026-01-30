# Animation Viewer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an animation viewer to the editor preview panel for QA, iteration, and debugging of skeletal animations.

**Architecture:** Hybrid rendering (Blender generates preview GLB, three.js plays it interactively). Supports both spec-driven preview and exported GLB viewing with full debug controls (play/pause, frame-stepping, timeline scrubbing, bone inspection).

**Tech Stack:** TypeScript (three.js for 3D rendering), Rust (Blender backend for preview generation), Tauri IPC

---

## Task 1: Backend Animation Preview Generation

**Files:**
- Create: `crates/speccade-editor/src/preview/animation.rs`
- Modify: `crates/speccade-editor/src/preview/mod.rs:6-10`
- Modify: `crates/speccade-editor/src/commands/generate.rs:77-80`

### Step 1: Write the failing test for animation preview

Create the test file structure:

```rust
// In crates/speccade-editor/src/preview/animation.rs

//! Animation preview generation.
//!
//! Generates skeletal animation previews as GLB files with embedded animation tracks.
//! Uses balanced quality settings: full rig accuracy, reduced mesh detail.

use super::{PreviewQuality, PreviewResult, PreviewSettings};
use crate::commands::lint::lint_asset_bytes;
use speccade_spec::Spec;

/// Animation-specific preview settings.
#[derive(Debug, Clone)]
pub struct AnimationPreviewSettings {
    /// Base preview settings.
    pub base: PreviewSettings,
    /// Whether to include mesh in preview (if mesh reference provided).
    pub include_mesh: bool,
}

impl Default for AnimationPreviewSettings {
    fn default() -> Self {
        Self {
            base: PreviewSettings::default(),
            include_mesh: true,
        }
    }
}

/// Generate an animation preview from a spec.
///
/// This generates a GLB with embedded animation tracks suitable for playback in three.js.
/// The mesh is reduced quality (LOD proxy) but the rig/bone transforms are full fidelity.
pub fn generate_animation_preview(spec: &Spec, settings: &PreviewSettings) -> PreviewResult {
    // Check if spec has a recipe
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => return PreviewResult::failure("animation", "No recipe defined"),
    };

    // Only handle animation recipes
    let is_animation_recipe = recipe.kind.starts_with("skeletal_animation.");
    if !is_animation_recipe {
        return PreviewResult::failure(
            "animation",
            format!("Recipe kind '{}' is not an animation recipe", recipe.kind),
        );
    }

    // Create a temporary directory for preview generation
    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return PreviewResult::failure("animation", format!("Failed to create temp dir: {}", e))
        }
    };

    let tmp_path = tmp_dir.path();
    let spec_path = tmp_path.join("preview.star");

    // Use the existing dispatch
    use speccade_cli::dispatch::dispatch_generate;

    match dispatch_generate(spec, tmp_path.to_str().unwrap(), &spec_path, None) {
        Ok(outputs) => {
            // Find the primary GLB output
            let glb_output = outputs
                .iter()
                .find(|o| matches!(o.format, speccade_spec::OutputFormat::Glb));

            match glb_output {
                Some(output) => {
                    // Read the generated GLB file
                    let glb_path = tmp_path.join(&output.path);
                    match std::fs::read(&glb_path) {
                        Ok(glb_bytes) => {
                            // Run lint on the generated animation
                            let lint_result = lint_asset_bytes(&glb_path, &glb_bytes, Some(spec));

                            // Extract animation metadata
                            let metadata = extract_animation_metadata(&glb_bytes, spec);

                            PreviewResult::success_with_quality(
                                "animation",
                                glb_bytes,
                                "model/gltf-binary",
                                metadata,
                                PreviewQuality::Full, // Animation previews are always full rig quality
                                false,                // No refinement needed
                            )
                            .with_lint(lint_result)
                        }
                        Err(e) => {
                            PreviewResult::failure("animation", format!("Failed to read GLB: {}", e))
                        }
                    }
                }
                None => PreviewResult::failure("animation", "No GLB output generated"),
            }
        }
        Err(e) => PreviewResult::failure("animation", format!("Generation failed: {}", e)),
    }
}

/// Extract metadata from an animation GLB file.
fn extract_animation_metadata(glb_bytes: &[u8], spec: &Spec) -> serde_json::Value {
    // Try to parse the GLB to extract animation info
    match gltf::Glb::from_slice(glb_bytes) {
        Ok(glb) => {
            match gltf::Gltf::from_slice(&glb.json) {
                Ok(gltf) => {
                    let mut animations = Vec::new();
                    let mut total_duration = 0.0f32;
                    let mut bone_count = 0u32;

                    // Count bones from skins
                    for skin in gltf.skins() {
                        bone_count = bone_count.max(skin.joints().count() as u32);
                    }

                    // Get animation info
                    for anim in gltf.animations() {
                        let name = anim.name().unwrap_or("Unnamed").to_string();
                        let mut duration = 0.0f32;
                        let mut channel_count = 0u32;

                        for channel in anim.channels() {
                            channel_count += 1;
                            let sampler = channel.sampler();
                            let input_accessor = sampler.input();
                            if let (Some(min), Some(max)) = (input_accessor.min(), input_accessor.max()) {
                                if let (Some(max_time), Some(min_time)) = (
                                    max.as_array().and_then(|a| a.first()).and_then(|v| v.as_f64()),
                                    min.as_array().and_then(|a| a.first()).and_then(|v| v.as_f64()),
                                ) {
                                    duration = duration.max((max_time - min_time) as f32);
                                }
                            }
                        }

                        total_duration = total_duration.max(duration);
                        animations.push(serde_json::json!({
                            "name": name,
                            "duration": duration,
                            "channels": channel_count,
                        }));
                    }

                    // Extract keyframe count from spec if available
                    let keyframe_count = extract_keyframe_count_from_spec(spec);

                    serde_json::json!({
                        "bone_count": bone_count,
                        "duration_seconds": total_duration,
                        "animations": animations,
                        "keyframe_count": keyframe_count,
                    })
                }
                Err(_) => serde_json::json!({
                    "parse_error": "Failed to parse GLTF JSON"
                }),
            }
        }
        Err(_) => serde_json::json!({
            "parse_error": "Failed to parse GLB"
        }),
    }
}

/// Extract keyframe count from spec recipe params.
fn extract_keyframe_count_from_spec(spec: &Spec) -> Option<u32> {
    let recipe = spec.recipe.as_ref()?;
    let params = recipe.params.as_object()?;

    // Try keyframes array
    if let Some(keyframes) = params.get("keyframes") {
        if let Some(arr) = keyframes.as_array() {
            return Some(arr.len() as u32);
        }
    }

    // Try phases array (for rigged animations)
    if let Some(phases) = params.get("phases") {
        if let Some(arr) = phases.as_array() {
            // Count keyframes across all phases
            let mut count = 0u32;
            for phase in arr {
                if let Some(kfs) = phase.get("keyframes").and_then(|v| v.as_array()) {
                    count += kfs.len() as u32;
                }
            }
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe};

    #[test]
    fn test_animation_preview_no_recipe() {
        let spec = Spec::builder("test-anim", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Glb, "test.glb"))
            .build();

        let settings = PreviewSettings::default();
        let result = generate_animation_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_animation_preview_wrong_recipe_type() {
        let recipe = Recipe::new("audio_v1", serde_json::json!({}));
        let spec = Spec::builder("test-anim", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(recipe)
            .build();

        let settings = PreviewSettings::default();
        let result = generate_animation_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not an animation recipe"));
    }
}
```

### Step 2: Run test to verify it compiles

Run: `cargo test -p speccade-editor preview::animation --no-run`
Expected: Compilation succeeds (module not wired up yet, will fail)

### Step 3: Wire up the animation module

Modify `crates/speccade-editor/src/preview/mod.rs`:

```rust
// Add after line 9 (after music module):
pub mod animation;
```

### Step 4: Wire up the generate command

Modify `crates/speccade-editor/src/commands/generate.rs` lines 77-80:

Replace:
```rust
speccade_spec::AssetType::SkeletalAnimation => {
    // Animation preview is not yet implemented
    PreviewResult::failure("animation", "Animation preview not yet implemented")
}
```

With:
```rust
speccade_spec::AssetType::SkeletalAnimation => {
    preview::animation::generate_animation_preview(&spec, &settings)
}
```

### Step 5: Run tests to verify

Run: `cargo test -p speccade-editor preview::animation`
Expected: Tests pass

### Step 6: Commit

```bash
git add crates/speccade-editor/src/preview/animation.rs crates/speccade-editor/src/preview/mod.rs crates/speccade-editor/src/commands/generate.rs
git commit -m "$(cat <<'EOF'
feat(editor): add animation preview generation backend

Wire up skeletal animation preview generation using the existing
Blender dispatch pipeline. Extracts animation metadata (bone count,
duration, keyframe count) from the generated GLB.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Frontend AnimationPreview Component - Basic Playback

**Files:**
- Create: `editor/src/components/AnimationPreview.ts`

### Step 1: Create the basic AnimationPreview component

```typescript
// editor/src/components/AnimationPreview.ts

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";
import { loadInspectSection, saveInspectSection } from "../lib/storage";

/** Render mode for animation preview. */
export type RenderMode = "lit" | "unlit" | "matcap" | "albedo" | "wireframe";

/** Preview mode: spec-driven or exported GLB. */
export type PreviewMode = "spec" | "exported";

/** Playback state. */
export type PlaybackState = "stopped" | "playing" | "paused";

/** Animation metadata from backend. */
export interface AnimationMetadata {
  bone_count?: number;
  duration_seconds?: number;
  keyframe_count?: number;
  animations?: Array<{
    name: string;
    duration: number;
    channels: number;
  }>;
}

/** Inspector state persisted to localStorage. */
type AnimationInspectorStateV1 = {
  v: 1;
  renderMode: RenderMode;
  previewMode: PreviewMode;
  showBones: boolean;
  playbackSpeed: number;
  loopEnabled: boolean;
  loopStart: number;
  loopEnd: number;
};

const STORAGE_KEY = "speccade:animation_preview:v1";

/**
 * Animation preview component using three.js.
 *
 * Renders animated GLB models with playback controls, bone visualization,
 * and frame-by-frame stepping for debugging.
 */
export class AnimationPreview {
  private container: HTMLElement;
  private wrapper: HTMLDivElement;

  // Three.js core
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private controls: OrbitControls;
  private clock: THREE.Clock;

  // Animation
  private mixer: THREE.AnimationMixer | null = null;
  private currentAction: THREE.AnimationAction | null = null;
  private currentGltf: GLTF | null = null;
  private currentModel: THREE.Group | null = null;

  // Bone visualization
  private boneHelpers: THREE.Object3D[] = [];
  private selectedBone: THREE.Bone | null = null;

  // State
  private playbackState: PlaybackState = "stopped";
  private playbackSpeed = 1.0;
  private loopEnabled = true;
  private loopStart = 0;
  private loopEnd = 1;
  private showBones = false;
  private renderMode: RenderMode = "lit";
  private previewMode: PreviewMode = "spec";
  private animationId: number | null = null;

  // UI elements
  private infoDiv: HTMLDivElement;
  private timelineDiv: HTMLDivElement;
  private playheadDiv: HTMLDivElement;
  private timeDisplay: HTMLDivElement;
  private playButton: HTMLButtonElement;
  private bonePanel: HTMLDivElement;
  private boneList: HTMLDivElement;
  private boneInspector: HTMLDivElement;

  // Metadata
  private metadata: AnimationMetadata | null = null;
  private filePathForState = "editor.star";

  constructor(container: HTMLElement) {
    this.container = container;
    this.clock = new THREE.Clock();

    // Create wrapper
    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      background: #1a1a1a;
    `;

    // Create scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x1a1a1a);

    // Create camera
    const width = container.clientWidth || 400;
    const height = container.clientHeight || 300;
    this.camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 1000);
    this.camera.position.set(3, 2, 3);

    // Create renderer
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(window.devicePixelRatio);

    // Create viewport container
    const viewportContainer = document.createElement("div");
    viewportContainer.style.cssText = `
      position: relative;
      flex: 1;
      min-height: 200px;
    `;
    viewportContainer.appendChild(this.renderer.domElement);

    // Create info overlay
    this.infoDiv = document.createElement("div");
    this.infoDiv.style.cssText = `
      position: absolute;
      top: 8px;
      left: 8px;
      font-size: 11px;
      color: #999;
      pointer-events: none;
    `;
    this.infoDiv.textContent = "No animation loaded";
    viewportContainer.appendChild(this.infoDiv);

    this.wrapper.appendChild(viewportContainer);

    // Create controls
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;

    // Add grid helper
    const gridHelper = new THREE.GridHelper(10, 10, 0x444444, 0x222222);
    this.scene.add(gridHelper);

    // Add lights
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
    this.scene.add(ambientLight);
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(5, 10, 7);
    this.scene.add(directionalLight);

    // Create timeline
    this.timelineDiv = this.createTimeline();
    this.wrapper.appendChild(this.timelineDiv);

    // Create transport controls
    const transportDiv = this.createTransportControls();
    this.wrapper.appendChild(transportDiv);

    // Create bone panel (collapsed by default)
    this.bonePanel = this.createBonePanel();
    this.wrapper.appendChild(this.bonePanel);

    container.appendChild(this.wrapper);

    // Load persisted state
    this.loadSettings();

    // Handle resize
    const resizeObserver = new ResizeObserver(() => this.handleResize());
    resizeObserver.observe(container);

    // Start animation loop
    this.animate();
  }

  private createTimeline(): HTMLDivElement {
    const timeline = document.createElement("div");
    timeline.style.cssText = `
      height: 40px;
      background: #252525;
      border-top: 1px solid #333;
      position: relative;
      cursor: pointer;
    `;

    // Playhead
    this.playheadDiv = document.createElement("div");
    this.playheadDiv.style.cssText = `
      position: absolute;
      top: 0;
      bottom: 0;
      width: 2px;
      background: #ffcc00;
      left: 0;
      pointer-events: none;
    `;
    timeline.appendChild(this.playheadDiv);

    // Time display
    this.timeDisplay = document.createElement("div");
    this.timeDisplay.style.cssText = `
      position: absolute;
      bottom: 4px;
      right: 8px;
      font-size: 10px;
      color: #888;
    `;
    this.timeDisplay.textContent = "0:00.000 / 0:00.000";
    timeline.appendChild(this.timeDisplay);

    // Click to seek
    timeline.addEventListener("click", (e) => {
      const rect = timeline.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const progress = x / rect.width;
      this.seekToProgress(progress);
    });

    return timeline;
  }

  private createTransportControls(): HTMLDivElement {
    const transport = document.createElement("div");
    transport.style.cssText = `
      display: flex;
      gap: 8px;
      padding: 8px 12px;
      background: #1e1e1e;
      border-top: 1px solid #333;
      align-items: center;
      flex-wrap: wrap;
    `;

    const buttonStyle = `
      padding: 4px 12px;
      background: #333;
      color: #ccc;
      border: 1px solid #444;
      border-radius: 4px;
      cursor: pointer;
      font-size: 11px;
    `;

    // Jump to start
    const startBtn = document.createElement("button");
    startBtn.textContent = "|◀";
    startBtn.style.cssText = buttonStyle;
    startBtn.onclick = () => this.seekToStart();
    transport.appendChild(startBtn);

    // Step back
    const stepBackBtn = document.createElement("button");
    stepBackBtn.textContent = "◀";
    stepBackBtn.style.cssText = buttonStyle;
    stepBackBtn.onclick = () => this.stepFrame(-1);
    transport.appendChild(stepBackBtn);

    // Play/Pause
    this.playButton = document.createElement("button");
    this.playButton.textContent = "▶";
    this.playButton.style.cssText = buttonStyle + "min-width: 50px;";
    this.playButton.onclick = () => this.togglePlayPause();
    transport.appendChild(this.playButton);

    // Step forward
    const stepFwdBtn = document.createElement("button");
    stepFwdBtn.textContent = "▶";
    stepFwdBtn.style.cssText = buttonStyle;
    stepFwdBtn.onclick = () => this.stepFrame(1);
    transport.appendChild(stepFwdBtn);

    // Jump to end
    const endBtn = document.createElement("button");
    endBtn.textContent = "▶|";
    endBtn.style.cssText = buttonStyle;
    endBtn.onclick = () => this.seekToEnd();
    transport.appendChild(endBtn);

    // Spacer
    const spacer = document.createElement("div");
    spacer.style.flex = "1";
    transport.appendChild(spacer);

    // Speed selector
    const speedLabel = document.createElement("span");
    speedLabel.textContent = "Speed:";
    speedLabel.style.cssText = "font-size: 11px; color: #888;";
    transport.appendChild(speedLabel);

    const speedSelect = document.createElement("select");
    speedSelect.style.cssText = `
      padding: 2px 6px;
      background: #333;
      color: #ccc;
      border: 1px solid #444;
      border-radius: 3px;
      font-size: 11px;
    `;
    [0.25, 0.5, 1, 2].forEach((speed) => {
      const opt = document.createElement("option");
      opt.value = String(speed);
      opt.textContent = `${speed}x`;
      if (speed === this.playbackSpeed) opt.selected = true;
      speedSelect.appendChild(opt);
    });
    speedSelect.onchange = () => {
      this.playbackSpeed = Number(speedSelect.value);
      if (this.currentAction) {
        this.currentAction.timeScale = this.playbackSpeed;
      }
      this.saveSettings();
    };
    transport.appendChild(speedSelect);

    // Bones toggle
    const bonesBtn = document.createElement("button");
    bonesBtn.textContent = "🦴 Bones";
    bonesBtn.style.cssText = buttonStyle;
    bonesBtn.onclick = () => {
      this.showBones = !this.showBones;
      this.updateBoneVisibility();
      this.bonePanel.style.display = this.showBones ? "flex" : "none";
      this.saveSettings();
    };
    transport.appendChild(bonesBtn);

    return transport;
  }

  private createBonePanel(): HTMLDivElement {
    const panel = document.createElement("div");
    panel.style.cssText = `
      display: none;
      flex-direction: row;
      height: 150px;
      background: #1e1e1e;
      border-top: 1px solid #333;
    `;

    // Bone list (left)
    this.boneList = document.createElement("div");
    this.boneList.style.cssText = `
      width: 200px;
      overflow-y: auto;
      border-right: 1px solid #333;
      font-size: 11px;
    `;
    panel.appendChild(this.boneList);

    // Bone inspector (right)
    this.boneInspector = document.createElement("div");
    this.boneInspector.style.cssText = `
      flex: 1;
      padding: 8px;
      font-size: 11px;
      color: #aaa;
      overflow-y: auto;
    `;
    this.boneInspector.textContent = "Select a bone to inspect";
    panel.appendChild(this.boneInspector);

    return panel;
  }

  /**
   * Load a GLB animation from base64 data.
   */
  async loadGLB(base64Data: string, metadata?: AnimationMetadata, filePath?: string): Promise<void> {
    this.filePathForState = filePath || "editor.star";
    this.metadata = metadata || null;

    // Remove existing model
    if (this.currentModel) {
      this.scene.remove(this.currentModel);
      this.currentModel = null;
    }

    // Stop current animation
    if (this.mixer) {
      this.mixer.stopAllAction();
      this.mixer = null;
    }
    this.currentAction = null;
    this.currentGltf = null;
    this.clearBoneHelpers();

    // Decode base64
    const binaryString = atob(base64Data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }

    // Load GLB
    const loader = new GLTFLoader();
    return new Promise((resolve, reject) => {
      loader.parse(
        bytes.buffer,
        "",
        (gltf) => {
          this.currentGltf = gltf;
          this.currentModel = gltf.scene;
          this.scene.add(gltf.scene);

          // Setup animation mixer
          this.mixer = new THREE.AnimationMixer(gltf.scene);

          // Play first animation if available
          if (gltf.animations.length > 0) {
            this.currentAction = this.mixer.clipAction(gltf.animations[0]);
            this.currentAction.timeScale = this.playbackSpeed;
            this.currentAction.loop = this.loopEnabled ? THREE.LoopRepeat : THREE.LoopOnce;
            this.currentAction.clampWhenFinished = true;
          }

          // Auto-fit camera
          this.fitCameraToModel(gltf.scene);

          // Build bone list
          this.buildBoneList(gltf.scene);

          // Update bone visibility
          this.updateBoneVisibility();

          // Update info display
          this.updateInfoDisplay();

          resolve();
        },
        (error) => {
          reject(new Error(`Failed to parse GLB: ${error}`));
        }
      );
    });
  }

  private fitCameraToModel(model: THREE.Object3D): void {
    const box = new THREE.Box3().setFromObject(model);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());

    const maxDim = Math.max(size.x, size.y, size.z);
    const fov = this.camera.fov * (Math.PI / 180);
    const distance = (maxDim / (2 * Math.tan(fov / 2))) * 1.5;

    this.camera.position.set(
      center.x + distance * 0.7,
      center.y + distance * 0.5,
      center.z + distance * 0.7
    );
    this.controls.target.copy(center);
    this.controls.update();
  }

  private buildBoneList(scene: THREE.Object3D): void {
    this.boneList.innerHTML = "";
    const bones: THREE.Bone[] = [];

    scene.traverse((obj) => {
      if ((obj as THREE.Bone).isBone) {
        bones.push(obj as THREE.Bone);
      }
    });

    bones.forEach((bone) => {
      const item = document.createElement("div");
      item.style.cssText = `
        padding: 4px 8px;
        cursor: pointer;
        border-bottom: 1px solid #333;
      `;
      item.textContent = bone.name || "Unnamed";
      item.onclick = () => this.selectBone(bone);
      this.boneList.appendChild(item);
    });
  }

  private selectBone(bone: THREE.Bone): void {
    this.selectedBone = bone;
    this.updateBoneInspector();
  }

  private updateBoneInspector(): void {
    if (!this.selectedBone) {
      this.boneInspector.textContent = "Select a bone to inspect";
      return;
    }

    const bone = this.selectedBone;
    const pos = bone.position;
    const rot = bone.rotation;
    const scale = bone.scale;

    this.boneInspector.innerHTML = `
      <div style="margin-bottom: 8px; font-weight: bold; color: #fff;">${bone.name || "Unnamed"}</div>
      <div style="margin-bottom: 4px;">Position (local)</div>
      <div style="color: #6cf;">X: ${pos.x.toFixed(3)} Y: ${pos.y.toFixed(3)} Z: ${pos.z.toFixed(3)}</div>
      <div style="margin-top: 8px; margin-bottom: 4px;">Rotation (euler)</div>
      <div style="color: #fc6;">X: ${(rot.x * 180 / Math.PI).toFixed(1)}° Y: ${(rot.y * 180 / Math.PI).toFixed(1)}° Z: ${(rot.z * 180 / Math.PI).toFixed(1)}°</div>
      <div style="margin-top: 8px; margin-bottom: 4px;">Scale</div>
      <div style="color: #6f6;">X: ${scale.x.toFixed(3)} Y: ${scale.y.toFixed(3)} Z: ${scale.z.toFixed(3)}</div>
    `;
  }

  private updateBoneVisibility(): void {
    this.clearBoneHelpers();

    if (!this.showBones || !this.currentModel) return;

    this.currentModel.traverse((obj) => {
      if ((obj as THREE.Bone).isBone) {
        const bone = obj as THREE.Bone;
        const helper = new THREE.AxesHelper(0.1);
        bone.add(helper);
        this.boneHelpers.push(helper);
      }
    });
  }

  private clearBoneHelpers(): void {
    this.boneHelpers.forEach((helper) => {
      helper.parent?.remove(helper);
    });
    this.boneHelpers = [];
  }

  private updateInfoDisplay(): void {
    if (!this.metadata && !this.currentGltf) {
      this.infoDiv.textContent = "No animation loaded";
      return;
    }

    const parts: string[] = [];

    if (this.metadata?.bone_count) {
      parts.push(`${this.metadata.bone_count} bones`);
    }

    if (this.currentGltf?.animations.length) {
      const clip = this.currentGltf.animations[0];
      parts.push(`${clip.duration.toFixed(2)}s`);
    }

    if (this.metadata?.keyframe_count) {
      parts.push(`${this.metadata.keyframe_count} keyframes`);
    }

    this.infoDiv.textContent = parts.join(" • ") || "Animation loaded";
  }

  // Playback controls
  togglePlayPause(): void {
    if (this.playbackState === "playing") {
      this.pause();
    } else {
      this.play();
    }
  }

  play(): void {
    if (!this.currentAction) return;
    this.currentAction.paused = false;
    this.currentAction.play();
    this.playbackState = "playing";
    this.playButton.textContent = "||";
    this.clock.start();
  }

  pause(): void {
    if (!this.currentAction) return;
    this.currentAction.paused = true;
    this.playbackState = "paused";
    this.playButton.textContent = "▶";
  }

  stop(): void {
    if (!this.currentAction) return;
    this.currentAction.stop();
    this.playbackState = "stopped";
    this.playButton.textContent = "▶";
  }

  seekToStart(): void {
    if (!this.currentAction) return;
    this.currentAction.time = 0;
    this.mixer?.update(0);
    this.updateTimelineUI();
  }

  seekToEnd(): void {
    if (!this.currentAction || !this.currentGltf?.animations[0]) return;
    this.currentAction.time = this.currentGltf.animations[0].duration;
    this.mixer?.update(0);
    this.updateTimelineUI();
  }

  seekToProgress(progress: number): void {
    if (!this.currentAction || !this.currentGltf?.animations[0]) return;
    const duration = this.currentGltf.animations[0].duration;
    this.currentAction.time = progress * duration;
    this.mixer?.update(0);
    this.updateTimelineUI();
  }

  stepFrame(direction: number): void {
    if (!this.currentAction || !this.currentGltf?.animations[0]) return;

    // Assume 30fps for frame stepping
    const fps = 30;
    const frameTime = 1 / fps;
    const duration = this.currentGltf.animations[0].duration;

    let newTime = this.currentAction.time + direction * frameTime;
    newTime = Math.max(0, Math.min(duration, newTime));

    this.currentAction.time = newTime;
    this.mixer?.update(0);
    this.updateTimelineUI();
  }

  private updateTimelineUI(): void {
    if (!this.currentAction || !this.currentGltf?.animations[0]) return;

    const current = this.currentAction.time;
    const duration = this.currentGltf.animations[0].duration;
    const progress = duration > 0 ? current / duration : 0;

    this.playheadDiv.style.left = `${progress * 100}%`;
    this.timeDisplay.textContent = `${this.formatTime(current)} / ${this.formatTime(duration)}`;

    // Update bone inspector if a bone is selected
    if (this.selectedBone) {
      this.updateBoneInspector();
    }
  }

  private formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 1000);
    return `${mins}:${String(secs).padStart(2, "0")}.${String(ms).padStart(3, "0")}`;
  }

  private animate(): void {
    this.animationId = requestAnimationFrame(() => this.animate());

    const delta = this.clock.getDelta();

    if (this.mixer && this.playbackState === "playing") {
      this.mixer.update(delta);
      this.updateTimelineUI();
    }

    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }

  private handleResize(): void {
    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  clear(): void {
    this.stop();
    if (this.currentModel) {
      this.scene.remove(this.currentModel);
      this.currentModel = null;
    }
    this.mixer = null;
    this.currentAction = null;
    this.currentGltf = null;
    this.metadata = null;
    this.clearBoneHelpers();
    this.boneList.innerHTML = "";
    this.boneInspector.textContent = "Select a bone to inspect";
    this.infoDiv.textContent = "No animation loaded";
  }

  dispose(): void {
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
    }
    this.controls.dispose();
    this.renderer.dispose();
    this.clearBoneHelpers();
    if (this.wrapper.parentElement === this.container) {
      this.container.removeChild(this.wrapper);
    }
  }

  private loadSettings(): void {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const state: AnimationInspectorStateV1 = JSON.parse(raw);
        if (state.v === 1) {
          this.renderMode = state.renderMode || "lit";
          this.previewMode = state.previewMode || "spec";
          this.showBones = state.showBones ?? false;
          this.playbackSpeed = state.playbackSpeed ?? 1.0;
          this.loopEnabled = state.loopEnabled ?? true;
          this.loopStart = state.loopStart ?? 0;
          this.loopEnd = state.loopEnd ?? 1;
        }
      }
    } catch {
      // ignore
    }
  }

  private saveSettings(): void {
    const state: AnimationInspectorStateV1 = {
      v: 1,
      renderMode: this.renderMode,
      previewMode: this.previewMode,
      showBones: this.showBones,
      playbackSpeed: this.playbackSpeed,
      loopEnabled: this.loopEnabled,
      loopStart: this.loopStart,
      loopEnd: this.loopEnd,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  }
}
```

### Step 2: Verify TypeScript compiles

Run: `cd editor && npm run build`
Expected: Build succeeds

### Step 3: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add AnimationPreview component with basic playback

Three.js-based animation viewer with:
- GLB loading and playback via AnimationMixer
- Transport controls (play/pause, step frame, seek)
- Timeline with playhead and time display
- Bone list and inspector panel
- Orbit controls and auto-camera fitting

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Wire AnimationPreview into Editor

**Files:**
- Modify: `editor/src/components/Editor.ts` (add animation preview case)
- Modify: `editor/src/lib/preview-dispatcher.ts` (if exists, or similar)

### Step 1: Find where previews are dispatched

Search for where MeshPreview is instantiated and used. Wire AnimationPreview similarly for `SkeletalAnimation` asset type.

### Step 2: Add AnimationPreview import and instantiation

Add to Editor.ts imports:
```typescript
import { AnimationPreview, type AnimationMetadata } from "./AnimationPreview";
```

Add preview instance and wire it up similar to MeshPreview pattern.

### Step 3: Test in editor

Run: `cd editor && npm run tauri dev`
Expected: Animation assets show the new preview component

### Step 4: Commit

```bash
git add editor/src/components/Editor.ts
git commit -m "$(cat <<'EOF'
feat(editor): wire AnimationPreview into preview panel

Animation specs now show the animation viewer instead of
"not implemented" error.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Add Mode Toggle (Spec Preview / Exported GLB)

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Add mode toggle UI

Add toggle buttons to transport controls for switching between spec preview and exported GLB modes.

### Step 2: Implement mode switching logic

When in "exported" mode, load the GLB from the output path instead of generating a preview.

### Step 3: Test mode toggle

Verify both modes work and persist state.

### Step 4: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add spec/export mode toggle to animation viewer

Toggle between spec-driven preview (regenerates on spec change)
and exported GLB view (loads final export).

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Add Render Mode Controls

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Add render mode selector

Add dropdown for render modes (Lit, Unlit, Albedo, Matcap, Wireframe) matching MeshPreview.

### Step 2: Implement material switching

Apply materials based on selected render mode, reusing pattern from MeshPreview.

### Step 3: Test render modes

Verify each render mode works correctly.

### Step 4: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add render mode controls to animation viewer

Support Lit, Unlit, Albedo, Matcap, Wireframe render modes
matching mesh preview behavior.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Add Loop Region Controls

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Add loop region UI

Add draggable handles on timeline for loop in/out points, similar to AudioPreview.

### Step 2: Implement loop region logic

Clamp playback to loop region when enabled.

### Step 3: Test loop region

Verify loop region works and persists.

### Step 4: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add loop region controls to animation viewer

Draggable loop in/out points on timeline with toggle.
Useful for isolating problematic animation sections.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Add Keyframe Markers on Timeline

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Parse keyframe times from metadata

Extract keyframe times from the spec metadata.

### Step 2: Render keyframe markers

Draw ticks on timeline at keyframe positions.

### Step 3: Click-to-jump on keyframe markers

Clicking a marker jumps to that keyframe time.

### Step 4: Test keyframe markers

Verify markers appear and clicking works.

### Step 5: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add keyframe markers to animation timeline

Show tick marks at keyframe positions from spec.
Click markers to jump directly to keyframes.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Add Click-on-Mesh Bone Selection

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Add raycaster for bone selection

Implement raycasting to find nearest bone when clicking on mesh.

### Step 2: Highlight selected bone

Show selected bone differently in bone overlay.

### Step 3: Sync with bone list panel

Selection in viewport syncs with bone list panel selection.

### Step 4: Test click-to-select

Verify clicking on mesh selects nearest bone.

### Step 5: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add click-to-select bone in animation viewer

Click on mesh to select nearest bone. Selection syncs
between viewport and bone hierarchy panel.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Add Bone Hierarchy Tree View

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Build hierarchical bone tree

Parse bone parent-child relationships from skeleton.

### Step 2: Render collapsible tree

Replace flat list with indented tree with expand/collapse.

### Step 3: Add search/filter

Add text input to filter bones by name.

### Step 4: Test bone hierarchy

Verify tree shows correct hierarchy and search works.

### Step 5: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add bone hierarchy tree to animation viewer

Collapsible tree view with search/filter for navigating
complex rigs. Replaces flat bone list.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Add Camera Preset Buttons

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Add camera preset buttons

Add buttons for Front, Side, Back, Top, Three-quarter views.

### Step 2: Implement camera preset logic

Position camera based on preset, matching PreviewCameraAngle from spec.

### Step 3: Test camera presets

Verify each preset positions camera correctly.

### Step 4: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add camera preset buttons to animation viewer

Quick access to Front, Side, Back, Top, Three-quarter views
for inspecting animation from different angles.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Add Preview Caching

**Files:**
- Modify: `crates/speccade-editor/src/preview/animation.rs`

### Step 1: Add content hash caching

Hash spec content and cache preview GLB by hash.

### Step 2: Skip generation if cached

Return cached preview if spec unchanged.

### Step 3: Test caching

Verify second preview request is fast (cache hit).

### Step 4: Commit

```bash
git add crates/speccade-editor/src/preview/animation.rs
git commit -m "$(cat <<'EOF'
feat(editor): add content-hash caching for animation previews

Skip Blender regeneration if spec content unchanged.
Significantly speeds up repeated preview requests.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Add Auto-Regenerate on Spec Change

**Files:**
- Modify: `editor/src/components/AnimationPreview.ts`

### Step 1: Add debounced regeneration

Watch for spec changes and trigger preview regeneration with 500ms debounce.

### Step 2: Show "Generating..." status

Display status indicator during regeneration.

### Step 3: Test auto-regenerate

Verify editing spec triggers preview update.

### Step 4: Commit

```bash
git add editor/src/components/AnimationPreview.ts
git commit -m "$(cat <<'EOF'
feat(editor): add auto-regenerate on spec change

Debounced preview regeneration when animation spec
is modified. Shows "Generating..." status during update.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 13: Integration Testing

**Files:**
- Create: `editor/src/components/__tests__/AnimationPreview.test.ts` (if testing infra exists)

### Step 1: Test with golden animation specs

Load known-good animation specs and verify preview generates.

### Step 2: Test playback controls

Verify play/pause/step/seek work correctly.

### Step 3: Test bone inspection

Verify bone selection and transform readout.

### Step 4: Manual QA

Test full workflow with real animation specs.

### Step 5: Commit

```bash
git add editor/src/components/__tests__/AnimationPreview.test.ts
git commit -m "$(cat <<'EOF'
test(editor): add integration tests for AnimationPreview

Verify preview generation, playback controls, and bone
inspection work correctly.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Summary

This plan implements the animation viewer in 13 tasks:

1. **Backend**: Animation preview generation (Rust)
2. **Frontend**: Basic AnimationPreview component (TypeScript/three.js)
3. **Integration**: Wire into editor preview panel
4. **Mode toggle**: Spec preview vs exported GLB
5. **Render modes**: Lit/Unlit/Albedo/Matcap/Wireframe
6. **Loop region**: Draggable in/out points
7. **Keyframe markers**: Visual ticks on timeline
8. **Click-to-select**: Bone selection via mesh click
9. **Bone hierarchy**: Collapsible tree with search
10. **Camera presets**: Quick view buttons
11. **Caching**: Content-hash based preview caching
12. **Auto-regenerate**: Debounced spec change detection
13. **Testing**: Integration tests and manual QA

Each task is independent and commits incrementally. The feature can be shipped after Task 3 with basic functionality, then enhanced progressively.
