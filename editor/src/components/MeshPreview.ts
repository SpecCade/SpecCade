import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

/** Preview quality levels from the backend. */
export type PreviewQuality = "proxy" | "full";

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

    // Start animation loop
    this.animate();
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

          // Use metadata if provided, otherwise use counted values
          const displayTriangles = metadata?.triangles ?? Math.round(triangles);
          const originalTriangles = metadata?.original_triangles;

          this.updateInfo(displayTriangles, vertices, quality, canRefine, originalTriangles);

          // Auto-fit camera to model
          this.fitCameraToModel(gltf.scene);

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
}
