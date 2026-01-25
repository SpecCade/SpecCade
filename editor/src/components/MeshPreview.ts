import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

/**
 * Mesh preview component using three.js.
 *
 * Renders GLB models with orbit controls, grid, and lighting.
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
   * Load a GLB model from base64 data.
   */
  loadGLB(base64Data: string): Promise<void> {
    return new Promise((resolve, reject) => {
      // Remove existing model
      if (this.currentModel) {
        this.scene.remove(this.currentModel);
        this.currentModel = null;
      }

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
          this.updateInfo(Math.round(triangles), vertices);

          // Auto-fit camera to model
          this.fitCameraToModel(gltf.scene);

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
    if (this.currentModel) {
      this.scene.remove(this.currentModel);
      this.currentModel = null;
    }
    this.updateInfo();
  }

  /**
   * Dispose of the preview and release resources.
   */
  dispose(): void {
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
    }
    this.controls.dispose();
    this.renderer.dispose();
    this.container.removeChild(this.renderer.domElement);
  }
}
