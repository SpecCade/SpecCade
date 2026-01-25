# Editor Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete EDITOR-003 through EDITOR-006 polish items: external file watching, texture preview with zoom/pan/tiling, and quality/metadata indicators across all preview types.

**Architecture:** Three independent improvements to the existing Tauri editor: (1) Rust-side file watcher using `notify` crate with IPC events to frontend, (2) new `TexturePreview.ts` component with canvas-based zoom/pan/tiling, (3) metadata display and quality badges in all preview components.

**Tech Stack:** Tauri 2.x, TypeScript, notify crate (Rust), Canvas 2D API, three.js (existing)

---

## Task 1: Add File Watcher Backend (Rust)

**Files:**
- Create: `crates/speccade-editor/src/watcher.rs`
- Modify: `crates/speccade-editor/src/lib.rs`
- Modify: `crates/speccade-editor/Cargo.toml`

**Step 1: Add notify dependency**

In `crates/speccade-editor/Cargo.toml`, add to `[dependencies]`:

```toml
notify = "6.1"
```

**Step 2: Run cargo check to verify dependency**

Run: `cd /c/Development/nethercore-project/speccade && cargo check -p speccade-editor`
Expected: Compiles successfully

**Step 3: Create watcher module**

Create `crates/speccade-editor/src/watcher.rs`:

```rust
//! File watcher for external file changes.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

/// Event emitted when a watched file changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    /// The path that changed.
    pub path: String,
    /// The kind of change (created, modified, removed).
    pub kind: String,
}

/// State for the file watcher.
pub struct WatcherState {
    watcher: Option<RecommendedWatcher>,
    watched_path: Option<PathBuf>,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            watcher: None,
            watched_path: None,
        }
    }
}

/// Start watching a file for changes.
#[tauri::command]
pub fn watch_file<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    state: tauri::State<'_, std::sync::Mutex<WatcherState>>,
) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let mut state = state.lock().map_err(|e| e.to_string())?;

    // Stop existing watcher if any
    state.watcher = None;
    state.watched_path = None;

    // Create channel for debouncing
    let (tx, rx) = mpsc::channel();

    // Create watcher with debounce
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )
    .map_err(|e| e.to_string())?;

    // Watch the file's parent directory (notify requires directory watching on some platforms)
    let watch_path = path.parent().unwrap_or(&path);
    watcher
        .watch(watch_path, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    // Spawn thread to handle events and emit to frontend
    let watched_file = path.clone();
    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now();
        while let Ok(event) = rx.recv() {
            // Only emit for our specific file
            let is_our_file = event.paths.iter().any(|p| p == &watched_file);
            if !is_our_file {
                continue;
            }

            // Debounce: don't emit more than once per 100ms
            if last_emit.elapsed() < Duration::from_millis(100) {
                continue;
            }
            last_emit = std::time::Instant::now();

            let kind = match event.kind {
                notify::EventKind::Create(_) => "created",
                notify::EventKind::Modify(_) => "modified",
                notify::EventKind::Remove(_) => "removed",
                _ => continue,
            };

            let change_event = FileChangeEvent {
                path: watched_file.to_string_lossy().to_string(),
                kind: kind.to_string(),
            };

            let _ = app.emit("file-changed", change_event);
        }
    });

    state.watcher = Some(watcher);
    state.watched_path = Some(path);

    Ok(())
}

/// Stop watching the current file.
#[tauri::command]
pub fn unwatch_file(
    state: tauri::State<'_, std::sync::Mutex<WatcherState>>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.watcher = None;
    state.watched_path = None;
    Ok(())
}
```

**Step 4: Register watcher module and commands**

In `crates/speccade-editor/src/lib.rs`, add:

```rust
pub mod watcher;

pub use watcher::{watch_file, unwatch_file, WatcherState};
```

And update the plugin init:

```rust
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("speccade")
        .invoke_handler(tauri::generate_handler![
            eval_spec,
            validate_spec,
            generate_preview,
            watch_file,
            unwatch_file,
        ])
        .setup(|app, _api| {
            app.manage(std::sync::Mutex::new(WatcherState::default()));
            Ok(())
        })
        .build()
}
```

**Step 5: Run tests to verify compilation**

Run: `cd /c/Development/nethercore-project/speccade && cargo test -p speccade-editor`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/speccade-editor/
git commit -m "feat(editor): add file watcher backend with notify crate"
```

---

## Task 2: Add File Watcher Frontend Integration

**Files:**
- Modify: `editor/src/main.ts`

**Step 1: Add file watcher imports and listeners**

In `editor/src/main.ts`, add after the existing imports:

```typescript
import { listen } from "@tauri-apps/api/event";

interface FileChangeEvent {
  path: string;
  kind: string;
}

// Current watched file path
let currentWatchedPath: string | null = null;
```

**Step 2: Add watch/unwatch functions**

Add after the state variables:

```typescript
// Watch a file for external changes
async function watchFile(path: string): Promise<void> {
  if (currentWatchedPath === path) return;

  try {
    await invoke("plugin:speccade|watch_file", { path });
    currentWatchedPath = path;
  } catch (error) {
    console.error("Failed to watch file:", error);
  }
}

// Stop watching the current file
async function unwatchFile(): Promise<void> {
  if (!currentWatchedPath) return;

  try {
    await invoke("plugin:speccade|unwatch_file");
    currentWatchedPath = null;
  } catch (error) {
    console.error("Failed to unwatch file:", error);
  }
}
```

**Step 3: Set up event listener in init**

In the `init()` function, add before `initEditor()`:

```typescript
// Listen for file change events from backend
listen<FileChangeEvent>("file-changed", (event) => {
  const { path, kind } = event.payload;
  if (kind === "modified" && editor) {
    // Reload file content - for now just trigger re-evaluation
    // In future: prompt user or auto-reload
    updateStatus(`External change detected: ${path}`);
    evaluateSource();
  }
});
```

**Step 4: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 5: Commit**

```bash
git add editor/src/main.ts
git commit -m "feat(editor): add file watcher frontend integration"
```

---

## Task 3: Create TexturePreview Component

**Files:**
- Create: `editor/src/components/TexturePreview.ts`

**Step 1: Create the TexturePreview component**

Create `editor/src/components/TexturePreview.ts`:

```typescript
/**
 * Texture preview component with zoom, pan, and tiling support.
 */
export class TexturePreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private controlsDiv: HTMLDivElement;
  private infoDiv: HTMLDivElement;

  private image: HTMLImageElement | null = null;
  private zoom = 1;
  private panX = 0;
  private panY = 0;
  private tiling = false;
  private isPanning = false;
  private lastMouseX = 0;
  private lastMouseY = 0;

  constructor(container: HTMLElement) {
    this.container = container;

    // Create wrapper
    const wrapper = document.createElement("div");
    wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      gap: 8px;
      padding: 12px;
      box-sizing: border-box;
    `;

    // Create info bar
    this.infoDiv = document.createElement("div");
    this.infoDiv.style.cssText = `
      font-size: 11px;
      color: #999;
      display: flex;
      gap: 12px;
    `;
    this.infoDiv.innerHTML = `<span>No texture loaded</span>`;
    wrapper.appendChild(this.infoDiv);

    // Create canvas
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = `
      flex: 1;
      width: 100%;
      background: #1e1e1e;
      border-radius: 4px;
      cursor: grab;
      image-rendering: pixelated;
    `;
    wrapper.appendChild(this.canvas);
    this.ctx = this.canvas.getContext("2d")!;

    // Create controls
    this.controlsDiv = document.createElement("div");
    this.controlsDiv.style.cssText = `
      display: flex;
      gap: 8px;
      justify-content: center;
      align-items: center;
    `;

    const zoomOutBtn = this.createButton("-", () => this.setZoom(this.zoom / 1.5));
    const zoomInBtn = this.createButton("+", () => this.setZoom(this.zoom * 1.5));
    const resetBtn = this.createButton("Reset", () => this.resetView());
    const tileBtn = this.createButton("Tile", () => this.toggleTiling());
    tileBtn.id = "tile-btn";

    this.controlsDiv.appendChild(zoomOutBtn);
    this.controlsDiv.appendChild(zoomInBtn);
    this.controlsDiv.appendChild(resetBtn);
    this.controlsDiv.appendChild(tileBtn);

    wrapper.appendChild(this.controlsDiv);
    container.appendChild(wrapper);

    // Set up event listeners
    this.canvas.addEventListener("wheel", (e) => this.handleWheel(e));
    this.canvas.addEventListener("mousedown", (e) => this.handleMouseDown(e));
    this.canvas.addEventListener("mousemove", (e) => this.handleMouseMove(e));
    this.canvas.addEventListener("mouseup", () => this.handleMouseUp());
    this.canvas.addEventListener("mouseleave", () => this.handleMouseUp());

    // Handle resize
    const resizeObserver = new ResizeObserver(() => this.handleResize());
    resizeObserver.observe(container);
    this.handleResize();

    this.drawEmpty();
  }

  private createButton(text: string, onClick: () => void): HTMLButtonElement {
    const btn = document.createElement("button");
    btn.textContent = text;
    btn.style.cssText = `
      padding: 4px 12px;
      background: #444;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
    `;
    btn.onclick = onClick;
    return btn;
  }

  /**
   * Load texture from base64 data.
   */
  loadTexture(base64Data: string, mimeType: string, metadata?: Record<string, unknown>): Promise<void> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        this.image = img;
        this.resetView();
        this.updateInfo(metadata);
        this.draw();
        resolve();
      };
      img.onerror = () => reject(new Error("Failed to load texture"));
      img.src = `data:${mimeType};base64,${base64Data}`;
    });
  }

  private updateInfo(metadata?: Record<string, unknown>): void {
    if (!this.image) {
      this.infoDiv.innerHTML = `<span>No texture loaded</span>`;
      return;
    }

    const parts = [
      `<span>${this.image.width}×${this.image.height}</span>`,
      `<span>Zoom: ${Math.round(this.zoom * 100)}%</span>`,
    ];

    if (metadata) {
      if (metadata.tileable) {
        parts.push(`<span style="color: #4a9eff;">Tileable</span>`);
      }
    }

    parts.push(`<span style="color: #888;">Preview</span>`);

    this.infoDiv.innerHTML = parts.join("");
  }

  private setZoom(newZoom: number): void {
    this.zoom = Math.max(0.1, Math.min(10, newZoom));
    this.updateInfo();
    this.draw();
  }

  private resetView(): void {
    this.zoom = 1;
    this.panX = 0;
    this.panY = 0;
    this.updateInfo();
    this.draw();
  }

  private toggleTiling(): void {
    this.tiling = !this.tiling;
    const btn = this.controlsDiv.querySelector("#tile-btn") as HTMLButtonElement;
    if (btn) {
      btn.style.background = this.tiling ? "#007acc" : "#444";
    }
    this.draw();
  }

  private handleWheel(e: WheelEvent): void {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    this.setZoom(this.zoom * factor);
  }

  private handleMouseDown(e: MouseEvent): void {
    this.isPanning = true;
    this.lastMouseX = e.clientX;
    this.lastMouseY = e.clientY;
    this.canvas.style.cursor = "grabbing";
  }

  private handleMouseMove(e: MouseEvent): void {
    if (!this.isPanning) return;

    const dx = e.clientX - this.lastMouseX;
    const dy = e.clientY - this.lastMouseY;
    this.panX += dx;
    this.panY += dy;
    this.lastMouseX = e.clientX;
    this.lastMouseY = e.clientY;
    this.draw();
  }

  private handleMouseUp(): void {
    this.isPanning = false;
    this.canvas.style.cursor = "grab";
  }

  private handleResize(): void {
    this.canvas.width = this.container.clientWidth - 24;
    this.canvas.height = Math.max(100, this.container.clientHeight - 80);
    this.draw();
  }

  private drawEmpty(): void {
    const { width, height } = this.canvas;
    this.ctx.fillStyle = "#1e1e1e";
    this.ctx.fillRect(0, 0, width, height);

    // Draw checkerboard pattern
    const size = 16;
    for (let y = 0; y < height; y += size) {
      for (let x = 0; x < width; x += size) {
        const isLight = ((x / size) + (y / size)) % 2 === 0;
        this.ctx.fillStyle = isLight ? "#2a2a2a" : "#222";
        this.ctx.fillRect(x, y, size, size);
      }
    }

    this.ctx.fillStyle = "#666";
    this.ctx.font = "12px sans-serif";
    this.ctx.textAlign = "center";
    this.ctx.fillText("No texture loaded", width / 2, height / 2);
  }

  private draw(): void {
    if (!this.image) {
      this.drawEmpty();
      return;
    }

    const { width, height } = this.canvas;

    // Draw checkerboard background
    const size = 16;
    for (let y = 0; y < height; y += size) {
      for (let x = 0; x < width; x += size) {
        const isLight = ((x / size) + (y / size)) % 2 === 0;
        this.ctx.fillStyle = isLight ? "#2a2a2a" : "#222";
        this.ctx.fillRect(x, y, size, size);
      }
    }

    const imgW = this.image.width * this.zoom;
    const imgH = this.image.height * this.zoom;

    if (this.tiling) {
      // Draw tiled pattern
      const startX = ((this.panX % imgW) + imgW) % imgW - imgW;
      const startY = ((this.panY % imgH) + imgH) % imgH - imgH;

      for (let y = startY; y < height; y += imgH) {
        for (let x = startX; x < width; x += imgW) {
          this.ctx.drawImage(this.image, x, y, imgW, imgH);
        }
      }
    } else {
      // Draw single image centered with pan offset
      const x = (width - imgW) / 2 + this.panX;
      const y = (height - imgH) / 2 + this.panY;
      this.ctx.drawImage(this.image, x, y, imgW, imgH);
    }
  }

  /**
   * Clear the preview.
   */
  clear(): void {
    this.image = null;
    this.resetView();
    this.drawEmpty();
  }

  /**
   * Dispose of the preview.
   */
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
git add editor/src/components/TexturePreview.ts
git commit -m "feat(editor): add TexturePreview component with zoom/pan/tiling"
```

---

## Task 4: Integrate TexturePreview into Main

**Files:**
- Modify: `editor/src/main.ts`

**Step 1: Import TexturePreview**

Add to imports:

```typescript
import { TexturePreview } from "./components/TexturePreview";
```

**Step 2: Add state variable**

Add after `let audioPreview`:

```typescript
let texturePreview: TexturePreview | null = null;
```

**Step 3: Update clearPreviewComponents**

Update the function to include texture preview:

```typescript
function clearPreviewComponents(): void {
  if (meshPreview) {
    meshPreview.dispose();
    meshPreview = null;
  }
  if (audioPreview) {
    audioPreview.dispose();
    audioPreview = null;
  }
  if (texturePreview) {
    texturePreview.dispose();
    texturePreview = null;
  }
  previewContent.innerHTML = "";
}
```

**Step 4: Replace renderTexturePreview**

Replace the entire `renderTexturePreview` function:

```typescript
async function renderTexturePreview(source: string): Promise<void> {
  // Create texture preview if needed
  if (!texturePreview) {
    previewContent.innerHTML = "";
    texturePreview = new TexturePreview(previewContent);
  }

  try {
    updateStatus("Generating texture preview...");
    const result = await invoke<GeneratePreviewOutput>(
      "plugin:speccade|generate_preview",
      { source, filename: "editor.star" }
    );

    if (!result.compile_success) {
      updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
      return;
    }

    const preview = result.preview;
    if (preview?.success && preview.data) {
      await texturePreview.loadTexture(
        preview.data,
        preview.mime_type ?? "image/png",
        preview.metadata as Record<string, unknown> | undefined
      );
      updateStatus("Texture preview ready");
    } else {
      updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
    }
  } catch (error) {
    updateStatus(`Preview error: ${error}`);
  }
}
```

**Step 5: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 6: Commit**

```bash
git add editor/src/main.ts
git commit -m "feat(editor): integrate TexturePreview component"
```

---

## Task 5: Add Duration Display to AudioPreview

**Files:**
- Modify: `editor/src/components/AudioPreview.ts`

**Step 1: Add duration display element**

In the constructor, after creating the controls div, add a duration display. Find the line `wrapper.appendChild(controls);` and add before it:

```typescript
// Create duration display
const durationDiv = document.createElement("div");
durationDiv.id = "duration-display";
durationDiv.style.cssText = `
  font-size: 11px;
  color: #999;
  text-align: center;
`;
durationDiv.textContent = "0:00 / 0:00";
wrapper.appendChild(durationDiv);
```

**Step 2: Add updateDuration method**

Add after the `updateProgress` method:

```typescript
/**
 * Update the duration display.
 */
private updateDuration(current: number, total: number): void {
  const formatTime = (s: number) => {
    const mins = Math.floor(s / 60);
    const secs = Math.floor(s % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const display = this.container.querySelector("#duration-display");
  if (display) {
    display.textContent = `${formatTime(current)} / ${formatTime(total)}`;
  }
}
```

**Step 3: Update visualize to show duration**

In the `visualize` method, after `this.updateProgress(Math.min(progress, 1));`, add:

```typescript
this.updateDuration(elapsed, this.audioBuffer.duration);
```

**Step 4: Update loadWAV to show initial duration**

In `loadWAV`, after `this.drawWaveform();`, add:

```typescript
this.updateDuration(0, this.audioBuffer.duration);
```

**Step 5: Update stop to reset duration**

In `stop`, after `this.updateProgress(0);`, add:

```typescript
if (this.audioBuffer) {
  this.updateDuration(0, this.audioBuffer.duration);
}
```

**Step 6: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 7: Commit**

```bash
git add editor/src/components/AudioPreview.ts
git commit -m "feat(editor): add duration display to AudioPreview"
```

---

## Task 6: Add Quality Badge to MeshPreview

**Files:**
- Modify: `editor/src/components/MeshPreview.ts`

**Step 1: Add info display property and element**

After the `private animationId` property, add:

```typescript
private infoDiv: HTMLDivElement;
```

In the constructor, after creating the renderer and before `container.appendChild(this.renderer.domElement);`, add:

```typescript
// Create info overlay
this.infoDiv = document.createElement("div");
this.infoDiv.style.cssText = `
  position: absolute;
  top: 8px;
  left: 8px;
  font-size: 11px;
  color: #999;
  display: flex;
  gap: 12px;
  pointer-events: none;
`;
this.infoDiv.innerHTML = `<span>No mesh loaded</span>`;

// Wrap renderer in a positioned container
const rendererWrapper = document.createElement("div");
rendererWrapper.style.cssText = `
  position: relative;
  width: 100%;
  height: 100%;
`;
rendererWrapper.appendChild(this.renderer.domElement);
rendererWrapper.appendChild(this.infoDiv);
container.appendChild(rendererWrapper);
```

And remove the existing `container.appendChild(this.renderer.domElement);` line.

**Step 2: Add updateInfo method**

Add after `fitCameraToModel`:

```typescript
/**
 * Update the info display.
 */
updateInfo(triangles?: number, vertices?: number): void {
  const parts: string[] = [];

  if (triangles !== undefined) {
    parts.push(`<span>${triangles.toLocaleString()} tris</span>`);
  }
  if (vertices !== undefined) {
    parts.push(`<span>${vertices.toLocaleString()} verts</span>`);
  }

  parts.push(`<span style="color: #888;">Preview</span>`);

  this.infoDiv.innerHTML = parts.length > 1 ? parts.join("") : `<span>No mesh loaded</span>`;
}
```

**Step 3: Update loadGLB to count geometry**

In the `loadGLB` method, after `this.scene.add(gltf.scene);` and before `this.fitCameraToModel`, add:

```typescript
// Count triangles and vertices
let triangles = 0;
let vertices = 0;
gltf.scene.traverse((child) => {
  if ((child as THREE.Mesh).isMesh) {
    const mesh = child as THREE.Mesh;
    const geometry = mesh.geometry;
    if (geometry.index) {
      triangles += geometry.index.count / 3;
    } else if (geometry.attributes.position) {
      triangles += geometry.attributes.position.count / 3;
    }
    if (geometry.attributes.position) {
      vertices += geometry.attributes.position.count;
    }
  }
});
this.updateInfo(Math.round(triangles), vertices);
```

**Step 4: Clear info on dispose**

In `clear`, add after removing the model:

```typescript
this.updateInfo();
```

**Step 5: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 6: Commit**

```bash
git add editor/src/components/MeshPreview.ts
git commit -m "feat(editor): add geometry info and quality badge to MeshPreview"
```

---

## Task 7: Add Preview Quality Badge to Audio

**Files:**
- Modify: `editor/src/components/AudioPreview.ts`

**Step 1: Add sample rate info to duration display**

In the constructor, update the durationDiv creation to also show sample rate. Change the initial text to:

```typescript
durationDiv.textContent = "0:00 / 0:00 • Preview";
```

**Step 2: Update loadWAV to show sample rate**

In `loadWAV`, update the line that calls `updateDuration` to:

```typescript
this.updateDuration(0, this.audioBuffer.duration);
this.updateSampleInfo(this.audioBuffer.sampleRate, this.audioBuffer.numberOfChannels);
```

**Step 3: Add updateSampleInfo method**

Add after `updateDuration`:

```typescript
/**
 * Update sample rate info display.
 */
private updateSampleInfo(sampleRate: number, channels: number): void {
  const display = this.container.querySelector("#duration-display");
  if (display) {
    const current = display.textContent?.split(" • ")[0] || "0:00 / 0:00";
    const channelStr = channels === 1 ? "mono" : channels === 2 ? "stereo" : `${channels}ch`;
    display.textContent = `${current} • ${sampleRate}Hz ${channelStr} • Preview`;
  }
}
```

**Step 4: Verify TypeScript compiles**

Run: `cd /c/Development/nethercore-project/speccade/editor && npx tsc --noEmit`
Expected: No errors

**Step 5: Commit**

```bash
git add editor/src/components/AudioPreview.ts
git commit -m "feat(editor): add sample rate info and preview badge to AudioPreview"
```

---

## Task 8: Final Integration Test

**Files:**
- None (manual testing)

**Step 1: Build the editor**

Run: `cd /c/Development/nethercore-project/speccade/editor && npm run build`
Expected: Build completes successfully

**Step 2: Run Tauri dev mode**

Run: `cd /c/Development/nethercore-project/speccade/editor && npm run tauri dev`
Expected: Editor window opens

**Step 3: Test texture preview**

Enter a texture spec and verify:
- Texture renders with checkerboard background
- Zoom in/out with mouse wheel works
- Pan with click-drag works
- Tile button toggles tiling mode
- Info bar shows dimensions and "Preview" badge

**Step 4: Test audio preview**

Enter an audio spec and verify:
- Waveform displays
- Duration shows "0:00 / X:XX"
- Sample rate and channel info displays
- "Preview" badge shows
- Play/stop works

**Step 5: Test mesh preview**

Enter a mesh spec and verify:
- Mesh renders with orbit controls
- Triangle/vertex count shows in overlay
- "Preview" badge shows

**Step 6: Final commit**

```bash
git add -A
git commit -m "feat(editor): complete EDITOR-003 through EDITOR-006 polish

- Add file watcher backend with notify crate
- Add file watcher frontend integration
- Add TexturePreview component with zoom/pan/tiling
- Add duration display to AudioPreview
- Add geometry info to MeshPreview
- Add preview quality badges to all preview types"
```

---

## Summary

| Task | Description | Files Changed |
|------|-------------|---------------|
| 1 | File watcher backend | `watcher.rs`, `lib.rs`, `Cargo.toml` |
| 2 | File watcher frontend | `main.ts` |
| 3 | TexturePreview component | `TexturePreview.ts` (new) |
| 4 | Integrate TexturePreview | `main.ts` |
| 5 | AudioPreview duration | `AudioPreview.ts` |
| 6 | MeshPreview info | `MeshPreview.ts` |
| 7 | AudioPreview info | `AudioPreview.ts` |
| 8 | Integration test | Manual testing |

Total: 8 tasks, ~6 files modified, 1 new file
