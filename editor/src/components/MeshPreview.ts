import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

/** Preview quality levels from the backend. */
export type PreviewQuality = "proxy" | "full";

/** Render mode for mesh preview material layer. */
export type RenderMode = "lit" | "unlit" | "matcap";

/** Texture reference - discriminated union for texture sources. */
export type TextureRef =
  | null
  | { kind: "golden"; id: string }
  | { kind: "file"; path: string }
  | { kind: "spec_output"; spec_path: string; output_path: string };

/** Persisted preview material settings. */
export interface PreviewMaterialSettings {
  render_mode: RenderMode;
  texture_ref: TextureRef;
}

/** localStorage key for preview material settings. */
const STORAGE_KEY = "speccade:mesh_preview_material:v1";

/** Metadata returned with mesh previews. */
export interface MeshMetadata {
  triangles: number;
  vertices?: number;
  original_triangles?: number;
  is_proxy?: boolean;
  bounds?: {
    min: [number, number, number];
    max: [number, number, number];
  };
}

/** Callback for requesting full-quality refinement. */
export type RefineCallback = () => void;

/**
 * Mesh preview component using three.js.
 *
 * Renders GLB models with orbit controls, grid, and lighting.
 * Supports LOD proxy display with refinement on demand or idle timeout.
 */
export class MeshPreview {
  private container: HTMLElement;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private controls: OrbitControls;
  private currentModel: THREE.Group | null = null;
  private animationId: number | null = null;
  private infoDiv: HTMLDivElement;
  private refineButton: HTMLButtonElement | null = null;
  private idleTimer: number | null = null;
  private currentQuality: PreviewQuality = "full";
  private onRefine: RefineCallback | null = null;
  private autoRefineDelay = 2000; // 2 seconds idle before auto-refine

  // Material layer state
  private renderMode: RenderMode = "lit";
  private textureRef: TextureRef = null;
  private currentTexture: THREE.Texture | null = null;
  private originalMaterials: Map<THREE.Mesh, THREE.Material | THREE.Material[]> = new Map();
  private overrideMaterials: THREE.Material[] = [];

  // Material layer UI
  private controlsDiv: HTMLDivElement | null = null;
  private renderModeSelect: HTMLSelectElement | null = null;
  private textureSelect: HTMLSelectElement | null = null;
  private goldenTextures: Array<{ id: string; label: string; kind: string }> = [];

  constructor(container: HTMLElement) {
    this.container = container;

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
      align-items: center;
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

    // Setup interaction for auto-refine delay reset
    this.renderer.domElement.addEventListener("mousedown", () => this.resetIdleTimer());
    this.renderer.domElement.addEventListener("wheel", () => this.resetIdleTimer());

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

    // Handle resize
    const resizeObserver = new ResizeObserver(() => this.handleResize());
    resizeObserver.observe(container);

    // Load persisted material settings
    this.loadSettings();

    // Create material layer controls
    this.createMaterialControls(rendererWrapper);

    // Start animation loop
    this.animate();
  }

  /**
   * Create the material layer controls UI.
   */
  private createMaterialControls(wrapper: HTMLElement): void {
    const selectStyle = `
      font-size: 10px;
      padding: 2px 4px;
      background: #333;
      color: #ccc;
      border: 1px solid #555;
      border-radius: 3px;
      cursor: pointer;
      pointer-events: auto;
    `;

    const labelStyle = `
      font-size: 10px;
      color: #888;
    `;

    // Controls container (bottom-left)
    this.controlsDiv = document.createElement("div");
    this.controlsDiv.style.cssText = `
      position: absolute;
      bottom: 8px;
      left: 8px;
      display: flex;
      gap: 12px;
      align-items: center;
      pointer-events: none;
    `;

    // Render mode control
    const renderGroup = document.createElement("div");
    renderGroup.style.cssText = "display: flex; align-items: center; gap: 4px;";
    const renderLabel = document.createElement("span");
    renderLabel.textContent = "Render:";
    renderLabel.style.cssText = labelStyle;

    this.renderModeSelect = document.createElement("select");
    this.renderModeSelect.style.cssText = selectStyle;
    this.renderModeSelect.innerHTML = `
      <option value="lit">Lit</option>
      <option value="unlit">Unlit</option>
      <option value="matcap">Matcap</option>
    `;
    this.renderModeSelect.value = this.renderMode;
    this.renderModeSelect.addEventListener("change", () => {
      this.setRenderMode(this.renderModeSelect!.value as RenderMode);
    });

    renderGroup.appendChild(renderLabel);
    renderGroup.appendChild(this.renderModeSelect);

    // Texture control
    const textureGroup = document.createElement("div");
    textureGroup.style.cssText = "display: flex; align-items: center; gap: 4px;";
    const textureLabel = document.createElement("span");
    textureLabel.textContent = "Texture:";
    textureLabel.style.cssText = labelStyle;

    this.textureSelect = document.createElement("select");
    this.textureSelect.style.cssText = selectStyle;
    this.updateTextureSelectOptions();
    this.textureSelect.addEventListener("change", () => {
      this.handleTextureSelectChange(this.textureSelect!.value);
    });

    textureGroup.appendChild(textureLabel);
    textureGroup.appendChild(this.textureSelect);

    this.controlsDiv.appendChild(renderGroup);
    this.controlsDiv.appendChild(textureGroup);
    wrapper.appendChild(this.controlsDiv);

    // Load golden textures list asynchronously
    this.loadGoldenTexturesList();
  }

  /**
   * Update the texture select dropdown options.
   */
  private updateTextureSelectOptions(): void {
    if (!this.textureSelect) return;

    const options: string[] = ['<option value="none">None</option>'];

    // Golden textures
    for (const tex of this.goldenTextures) {
      options.push(`<option value="golden:${tex.id}">Golden: ${tex.label}</option>`);
    }

    // Special actions
    options.push('<option value="__file__">From PNG file...</option>');
    options.push('<option value="__spec__">From spec output...</option>');

    this.textureSelect.innerHTML = options.join("");

    // Restore selection
    if (this.textureRef === null) {
      this.textureSelect.value = "none";
    } else if (this.textureRef.kind === "golden") {
      this.textureSelect.value = `golden:${this.textureRef.id}`;
    }
    // file and spec_output don't have persistent select values,
    // they trigger file pickers
  }

  /**
   * Load the list of golden preview textures from backend.
   */
  private async loadGoldenTexturesList(): Promise<void> {
    try {
      // @ts-expect-error Tauri IPC
      const { invoke } = window.__TAURI__.core;
      this.goldenTextures = await invoke("plugin:speccade|list_golden_preview_textures");
      this.updateTextureSelectOptions();
    } catch (e) {
      console.warn("Failed to load golden textures list:", e);
    }
  }

  /**
   * Handle texture select dropdown change.
   */
  private async handleTextureSelectChange(value: string): Promise<void> {
    if (value === "none") {
      await this.setTextureRef(null);
    } else if (value.startsWith("golden:")) {
      const id = value.slice(7);
      await this.setTextureRef({ kind: "golden", id });
    } else if (value === "__file__") {
      await this.pickTextureFile();
      // Reset select to current ref
      this.updateTextureSelectOptions();
    } else if (value === "__spec__") {
      await this.pickSpecOutput();
      // Reset select to current ref
      this.updateTextureSelectOptions();
    }
  }

  /**
   * Open file picker for PNG texture.
   */
  private async pickTextureFile(): Promise<void> {
    try {
      // @ts-expect-error Tauri IPC
      const { open } = window.__TAURI__.dialog;
      const path = await open({
        filters: [{ name: "PNG Images", extensions: ["png"] }],
        multiple: false,
      });
      if (path && typeof path === "string") {
        await this.setTextureRef({ kind: "file", path });
      }
    } catch (e) {
      console.warn("File picker cancelled or failed:", e);
    }
  }

  /**
   * Open file picker for spec, then show output selection.
   */
  private async pickSpecOutput(): Promise<void> {
    try {
      // @ts-expect-error Tauri IPC
      const { open } = window.__TAURI__.dialog;
      // @ts-expect-error Tauri IPC
      const { invoke } = window.__TAURI__.core;

      const specPath = await open({
        filters: [{ name: "Starlark Specs", extensions: ["star"] }],
        multiple: false,
      });
      if (!specPath || typeof specPath !== "string") return;

      // Read spec source
      const source: string = await invoke("plugin:speccade|read_file", { path: specPath });

      // Eval spec to get outputs
      const evalResult = await invoke("plugin:speccade|eval_spec", {
        source,
        filename: specPath,
      });

      // Filter to PNG outputs
      const pngOutputs = (evalResult.outputs || []).filter(
        (o: { path: string; format: string }) => o.format === "png"
      );

      if (pngOutputs.length === 0) {
        console.warn("No PNG outputs declared in spec");
        return;
      }

      // If single output, use it directly
      if (pngOutputs.length === 1) {
        await this.setTextureRef({
          kind: "spec_output",
          spec_path: specPath,
          output_path: pngOutputs[0].path,
        });
        return;
      }

      // Multiple outputs - prompt user (simple prompt for now)
      const outputPath = prompt(
        `Select PNG output:\n${pngOutputs.map((o: { path: string }, i: number) => `${i + 1}. ${o.path}`).join("\n")}\n\nEnter number:`
      );
      if (outputPath) {
        const idx = parseInt(outputPath, 10) - 1;
        if (idx >= 0 && idx < pngOutputs.length) {
          await this.setTextureRef({
            kind: "spec_output",
            spec_path: specPath,
            output_path: pngOutputs[idx].path,
          });
        }
      }
    } catch (e) {
      console.warn("Spec output picker failed:", e);
    }
  }

  /**
   * Set the callback for refinement requests.
   */
  setRefineCallback(callback: RefineCallback | null): void {
    this.onRefine = callback;
  }

  /**
   * Set the auto-refine delay (in milliseconds).
   * Set to 0 to disable auto-refine.
   */
  setAutoRefineDelay(delay: number): void {
    this.autoRefineDelay = delay;
  }

  /**
   * Load a GLB model from base64 data.
   */
  loadGLB(
    base64Data: string,
    quality: PreviewQuality = "full",
    canRefine = false,
    metadata?: MeshMetadata
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      // Clear any pending idle timer
      this.clearIdleTimer();

      // Remove existing model
      if (this.currentModel) {
        this.scene.remove(this.currentModel);
        this.currentModel = null;
      }

      // Update quality state
      this.currentQuality = quality;

      // Decode base64
      const binaryString = atob(base64Data);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }

      // Load GLB
      const loader = new GLTFLoader();
      loader.parse(
        bytes.buffer,
        "",
        (gltf) => {
          this.currentModel = gltf.scene;
          this.scene.add(gltf.scene);

          // Clear and cache original materials
          this.originalMaterials.clear();
          this.disposeOverrideMaterials();

          // Count triangles and vertices, cache original materials
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
              // Cache original material
              this.originalMaterials.set(mesh, mesh.material);
            }
          });

          // Use metadata if provided, otherwise use counted values
          const displayTriangles = metadata?.triangles ?? Math.round(triangles);
          const originalTriangles = metadata?.original_triangles;

          this.updateInfo(displayTriangles, vertices, quality, canRefine, originalTriangles);

          // Auto-fit camera to model
          this.fitCameraToModel(gltf.scene);

          // Apply current render mode materials
          this.applyMaterials();

          // Start idle timer for auto-refine if this is a proxy
          if (canRefine && this.autoRefineDelay > 0) {
            this.startIdleTimer();
          }

          resolve();
        },
        (error) => {
          reject(new Error(`Failed to parse GLB: ${error}`));
        }
      );
    });
  }

  /**
   * Fit the camera to show the entire model.
   */
  private fitCameraToModel(model: THREE.Object3D): void {
    const box = new THREE.Box3().setFromObject(model);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());

    const maxDim = Math.max(size.x, size.y, size.z);
    const fov = this.camera.fov * (Math.PI / 180);
    const distance = maxDim / (2 * Math.tan(fov / 2)) * 1.5;

    this.camera.position.set(
      center.x + distance * 0.7,
      center.y + distance * 0.5,
      center.z + distance * 0.7
    );
    this.controls.target.copy(center);
    this.controls.update();
  }

  /**
   * Update the info display with quality badge and refine button.
   */
  updateInfo(
    triangles?: number,
    vertices?: number,
    quality: PreviewQuality = "full",
    canRefine = false,
    originalTriangles?: number
  ): void {
    const parts: string[] = [];

    if (triangles !== undefined) {
      let triText = `${triangles.toLocaleString()} tris`;
      // Show original count if this is a proxy
      if (quality === "proxy" && originalTriangles !== undefined) {
        triText += ` <span style="color: #666;">(of ${originalTriangles.toLocaleString()})</span>`;
      }
      parts.push(`<span>${triText}</span>`);
    }
    if (vertices !== undefined) {
      parts.push(`<span>${vertices.toLocaleString()} verts</span>`);
    }

    // Quality badge
    if (quality === "proxy") {
      parts.push(`<span style="
        background: #f59e0b;
        color: #000;
        padding: 1px 6px;
        border-radius: 3px;
        font-weight: 500;
        font-size: 10px;
      ">Preview</span>`);
    } else {
      parts.push(`<span style="
        background: #10b981;
        color: #000;
        padding: 1px 6px;
        border-radius: 3px;
        font-weight: 500;
        font-size: 10px;
      ">Full Quality</span>`);
    }

    this.infoDiv.innerHTML = parts.length > 0 ? parts.join("") : `<span>No mesh loaded</span>`;

    // Add or remove refine button
    this.updateRefineButton(canRefine);
  }

  /**
   * Update the refine button visibility.
   */
  private updateRefineButton(show: boolean): void {
    if (show && !this.refineButton) {
      this.refineButton = document.createElement("button");
      this.refineButton.textContent = "Refine";
      this.refineButton.style.cssText = `
        position: absolute;
        bottom: 8px;
        right: 8px;
        padding: 4px 12px;
        font-size: 11px;
        background: #3b82f6;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 500;
        transition: background 0.2s;
      `;
      this.refineButton.addEventListener("mouseenter", () => {
        if (this.refineButton) {
          this.refineButton.style.background = "#2563eb";
        }
      });
      this.refineButton.addEventListener("mouseleave", () => {
        if (this.refineButton) {
          this.refineButton.style.background = "#3b82f6";
        }
      });
      this.refineButton.addEventListener("click", () => {
        this.requestRefine();
      });

      // Find the renderer wrapper and add the button
      const wrapper = this.infoDiv.parentElement;
      if (wrapper) {
        wrapper.appendChild(this.refineButton);
      }
    } else if (!show && this.refineButton) {
      this.refineButton.remove();
      this.refineButton = null;
    }
  }

  /**
   * Request refinement to full quality.
   */
  requestRefine(): void {
    if (this.onRefine) {
      // Disable button while refining
      if (this.refineButton) {
        this.refineButton.disabled = true;
        this.refineButton.textContent = "Loading...";
        this.refineButton.style.background = "#6b7280";
      }
      this.clearIdleTimer();
      this.onRefine();
    }
  }

  /**
   * Start the idle timer for auto-refine.
   */
  private startIdleTimer(): void {
    this.clearIdleTimer();
    if (this.autoRefineDelay > 0 && this.currentQuality === "proxy") {
      this.idleTimer = window.setTimeout(() => {
        this.requestRefine();
      }, this.autoRefineDelay);
    }
  }

  /**
   * Reset the idle timer (called on user interaction).
   */
  private resetIdleTimer(): void {
    if (this.currentQuality === "proxy" && this.autoRefineDelay > 0) {
      this.startIdleTimer();
    }
  }

  /**
   * Clear the idle timer.
   */
  private clearIdleTimer(): void {
    if (this.idleTimer !== null) {
      window.clearTimeout(this.idleTimer);
      this.idleTimer = null;
    }
  }

  /**
   * Handle container resize.
   */
  private handleResize(): void {
    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  /**
   * Animation loop.
   */
  private animate(): void {
    this.animationId = requestAnimationFrame(() => this.animate());
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }

  /**
   * Clear the preview.
   */
  clear(): void {
    this.clearIdleTimer();
    if (this.currentModel) {
      this.scene.remove(this.currentModel);
      this.currentModel = null;
    }
    this.currentQuality = "full";
    this.updateRefineButton(false);
    this.infoDiv.innerHTML = `<span>No mesh loaded</span>`;

    // Clear material layer state
    this.originalMaterials.clear();
    this.disposeOverrideMaterials();
  }

  /**
   * Dispose of the preview and release resources.
   */
  dispose(): void {
    this.clearIdleTimer();
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
    }
    this.controls.dispose();
    this.renderer.dispose();
    this.updateRefineButton(false);

    // Dispose material layer resources
    this.originalMaterials.clear();
    this.disposeOverrideMaterials();
    if (this.currentTexture) {
      this.currentTexture.dispose();
      this.currentTexture = null;
    }

    // Find the wrapper element (parent of both renderer and infoDiv)
    const wrapper = this.infoDiv.parentElement;
    if (wrapper && wrapper.parentElement === this.container) {
      this.container.removeChild(wrapper);
    }
  }

  /**
   * Get the current quality level.
   */
  getQuality(): PreviewQuality {
    return this.currentQuality;
  }

  /**
   * Check if the current preview can be refined.
   */
  canRefine(): boolean {
    return this.currentQuality === "proxy" && this.onRefine !== null;
  }

  /**
   * Set the render mode and apply materials.
   */
  setRenderMode(mode: RenderMode): void {
    if (this.renderMode === mode) return;
    this.renderMode = mode;
    this.saveSettings();
    this.applyMaterials();

    // Update UI
    if (this.renderModeSelect) {
      this.renderModeSelect.value = mode;
    }
  }

  /**
   * Set the texture reference and load the texture.
   */
  async setTextureRef(ref: TextureRef): Promise<void> {
    this.textureRef = ref;
    this.saveSettings();

    // Dispose old texture
    if (this.currentTexture) {
      this.currentTexture.dispose();
      this.currentTexture = null;
    }

    // Load new texture if ref is not null
    if (ref !== null) {
      await this.loadTextureFromRef(ref);
    }

    // Apply materials with new texture
    this.applyMaterials();

    // Update UI
    this.updateTextureSelectOptions();
  }

  /**
   * Load texture data from a texture reference.
   */
  private async loadTextureFromRef(ref: TextureRef): Promise<void> {
    if (ref === null) return;

    try {
      // @ts-expect-error Tauri IPC
      const { invoke } = window.__TAURI__.core;
      let base64: string;

      if (ref.kind === "golden") {
        // Get golden texture source, then generate PNG
        const sourceData = await invoke("plugin:speccade|get_golden_preview_texture_source", {
          id: ref.id,
        });

        // Get the first PNG output path from the spec
        const outputPath = await this.getFirstPngOutput(sourceData.source, sourceData.filename);

        // Generate the texture using the same pipeline as regular preview
        const result = await invoke("plugin:speccade|generate_png_output_base64", {
          source: sourceData.source,
          filename: sourceData.filename,
          outputPath,
        });
        base64 = result.base64;
      } else if (ref.kind === "file") {
        // Read binary file
        const result = await invoke("plugin:speccade|read_binary_file_base64", {
          path: ref.path,
        });
        base64 = result.base64;
      } else if (ref.kind === "spec_output") {
        // Read spec source and generate specific output
        const source = await invoke("plugin:speccade|read_file", { path: ref.spec_path });
        const result = await invoke("plugin:speccade|generate_png_output_base64", {
          source,
          filename: ref.spec_path,
          outputPath: ref.output_path,
        });
        base64 = result.base64;
      } else {
        return;
      }

      // Convert base64 to texture
      this.currentTexture = await this.base64ToTexture(base64);
    } catch (e) {
      console.warn("Failed to load texture:", e);
      this.currentTexture = null;
    }
  }

  /**
   * Get the first PNG output path from a spec source (helper for golden textures).
   */
  private async getFirstPngOutput(source: string, filename: string): Promise<string> {
    try {
      // @ts-expect-error Tauri IPC
      const { invoke } = window.__TAURI__.core;
      const evalResult = await invoke("plugin:speccade|eval_spec", { source, filename });
      const pngOutputs = (evalResult.outputs || []).filter(
        (o: { path: string; format: string }) => o.format === "png"
      );
      if (pngOutputs.length > 0) {
        return pngOutputs[0].path;
      }
    } catch (e) {
      console.warn("Failed to get PNG output from spec:", e);
    }
    return "output.png"; // fallback
  }

  /**
   * Convert base64 PNG data to THREE.Texture.
   */
  private base64ToTexture(base64: string): Promise<THREE.Texture> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        const texture = new THREE.Texture(img);
        texture.needsUpdate = true;
        texture.colorSpace = THREE.SRGBColorSpace;
        resolve(texture);
      };
      img.onerror = () => reject(new Error("Failed to load texture image"));
      img.src = `data:image/png;base64,${base64}`;
    });
  }

  /**
   * Apply materials based on current render mode and texture.
   */
  private applyMaterials(): void {
    if (!this.currentModel) return;

    // Dispose old override materials
    this.disposeOverrideMaterials();

    if (this.renderMode === "lit") {
      // Restore original materials
      this.originalMaterials.forEach((material, mesh) => {
        mesh.material = material;
      });
    } else if (this.renderMode === "unlit") {
      // Apply MeshBasicMaterial with texture as map (UV0)
      this.originalMaterials.forEach((_, mesh) => {
        const mat = new THREE.MeshBasicMaterial({
          map: this.currentTexture,
          color: this.currentTexture ? 0xffffff : 0x888888,
        });
        this.overrideMaterials.push(mat);
        mesh.material = mat;
      });
    } else if (this.renderMode === "matcap") {
      // Apply MeshMatcapMaterial with texture as matcap
      this.originalMaterials.forEach((_, mesh) => {
        const mat = new THREE.MeshMatcapMaterial({
          matcap: this.currentTexture,
          color: 0xffffff,
        });
        this.overrideMaterials.push(mat);
        mesh.material = mat;
      });
    }
  }

  /**
   * Dispose override materials to prevent memory leaks.
   */
  private disposeOverrideMaterials(): void {
    for (const mat of this.overrideMaterials) {
      mat.dispose();
    }
    this.overrideMaterials = [];
  }

  /**
   * Load persisted material settings from localStorage.
   */
  private loadSettings(): void {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const settings: PreviewMaterialSettings = JSON.parse(raw);
        if (settings.render_mode) {
          this.renderMode = settings.render_mode;
        }
        if (settings.texture_ref !== undefined) {
          this.textureRef = settings.texture_ref;
        }
      }
    } catch {
      // Ignore parse errors, use defaults
    }
  }

  /**
   * Save current material settings to localStorage.
   */
  private saveSettings(): void {
    const settings: PreviewMaterialSettings = {
      render_mode: this.renderMode,
      texture_ref: this.textureRef,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }

  /**
   * Get the current render mode.
   */
  getRenderMode(): RenderMode {
    return this.renderMode;
  }

  /**
   * Get the current texture reference.
   */
  getTextureRef(): TextureRef {
    return this.textureRef;
  }
}
