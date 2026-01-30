import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";

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
  private infoDiv!: HTMLDivElement;
  private timelineDiv!: HTMLDivElement;
  private playheadDiv!: HTMLDivElement;
  private timeDisplay!: HTMLDivElement;
  private playButton!: HTMLButtonElement;
  private bonePanel!: HTMLDivElement;
  private boneList!: HTMLDivElement;
  private boneInspector!: HTMLDivElement;

  // Metadata
  private metadata: AnimationMetadata | null = null;

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
    startBtn.textContent = "|<";
    startBtn.style.cssText = buttonStyle;
    startBtn.onclick = () => this.seekToStart();
    transport.appendChild(startBtn);

    // Step back
    const stepBackBtn = document.createElement("button");
    stepBackBtn.textContent = "<";
    stepBackBtn.style.cssText = buttonStyle;
    stepBackBtn.onclick = () => this.stepFrame(-1);
    transport.appendChild(stepBackBtn);

    // Play/Pause
    this.playButton = document.createElement("button");
    this.playButton.textContent = "Play";
    this.playButton.style.cssText = buttonStyle + "min-width: 50px;";
    this.playButton.onclick = () => this.togglePlayPause();
    transport.appendChild(this.playButton);

    // Step forward
    const stepFwdBtn = document.createElement("button");
    stepFwdBtn.textContent = ">";
    stepFwdBtn.style.cssText = buttonStyle;
    stepFwdBtn.onclick = () => this.stepFrame(1);
    transport.appendChild(stepFwdBtn);

    // Jump to end
    const endBtn = document.createElement("button");
    endBtn.textContent = ">|";
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
    bonesBtn.textContent = "Bones";
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
  async loadGLB(base64Data: string, metadata?: AnimationMetadata, _filePath?: string): Promise<void> {
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
      <div style="color: #fc6;">X: ${(rot.x * 180 / Math.PI).toFixed(1)}deg Y: ${(rot.y * 180 / Math.PI).toFixed(1)}deg Z: ${(rot.z * 180 / Math.PI).toFixed(1)}deg</div>
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

    this.infoDiv.textContent = parts.join(" | ") || "Animation loaded";
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
    this.playButton.textContent = "Pause";
    this.clock.start();
  }

  pause(): void {
    if (!this.currentAction) return;
    this.currentAction.paused = true;
    this.playbackState = "paused";
    this.playButton.textContent = "Play";
  }

  stop(): void {
    if (!this.currentAction) return;
    this.currentAction.stop();
    this.playbackState = "stopped";
    this.playButton.textContent = "Play";
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
