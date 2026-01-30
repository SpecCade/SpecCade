import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";

/** Render mode for animation preview. */
export type RenderMode = "lit" | "unlit" | "matcap" | "albedo" | "wireframe";

/** Preview mode: spec-driven or exported GLB. */
export type PreviewMode = "spec" | "exported";

/** Playback state. */
export type PlaybackState = "stopped" | "playing" | "paused";

/** Camera preset view angle. */
export type CameraPreset = "front" | "side" | "back" | "top" | "three-quarter";

/** Animation metadata from backend. */
export interface AnimationMetadata {
  bone_count?: number;
  duration_seconds?: number;
  keyframe_count?: number;
  keyframe_times?: number[];
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
  private boneFilter = "";
  private expandedBones: Set<string> = new Set();

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

  // Callbacks
  private onModeChange: ((mode: PreviewMode) => void) | null = null;

  // Mode toggle buttons
  private specModeBtn!: HTMLButtonElement;
  private exportedModeBtn!: HTMLButtonElement;
  private renderModeSelect!: HTMLSelectElement;

  // Loop region elements
  private loopRegionDiv!: HTMLDivElement;
  private loopStartHandle!: HTMLDivElement;
  private loopEndHandle!: HTMLDivElement;
  private loopToggleBtn!: HTMLButtonElement;

  // Keyframe marker elements
  private keyframeMarkersContainer!: HTMLDivElement;
  private keyframeMarkers: HTMLDivElement[] = [];

  // Status overlay
  private statusOverlay!: HTMLDivElement;

  // Material layer state
  private originalMaterials: Map<THREE.Mesh, THREE.Material | THREE.Material[]> = new Map();
  private overrideMaterials: THREE.Material[] = [];

  // Raycasting for bone selection
  private raycaster: THREE.Raycaster;
  private bones: THREE.Bone[] = [];
  private selectedBoneHelper: THREE.AxesHelper | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.clock = new THREE.Clock();
    this.raycaster = new THREE.Raycaster();

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

    // Create status overlay (for "Generating..." state)
    this.statusOverlay = document.createElement("div");
    this.statusOverlay.style.cssText = `
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      display: none;
      align-items: center;
      justify-content: center;
      background: rgba(26, 26, 26, 0.85);
      font-size: 14px;
      color: #4a9eff;
      pointer-events: none;
    `;
    this.statusOverlay.innerHTML = `<span style="animation: pulse 1.5s infinite; @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }">Generating...</span>`;
    viewportContainer.appendChild(this.statusOverlay);

    this.wrapper.appendChild(viewportContainer);

    // Create controls
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;

    // Add click handler for bone selection
    this.renderer.domElement.addEventListener("click", (e) => this.handleViewportClick(e));

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

    // Update loop UI after settings load
    this.updateLoopRegionUI();
    this.updateLoopVisibility();

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

    // Keyframe markers container (behind loop region)
    this.keyframeMarkersContainer = document.createElement("div");
    this.keyframeMarkersContainer.style.cssText = `
      position: absolute;
      top: 0;
      bottom: 0;
      left: 0;
      right: 0;
      pointer-events: none;
    `;
    timeline.appendChild(this.keyframeMarkersContainer);

    // Loop region (behind playhead)
    this.loopRegionDiv = document.createElement("div");
    this.loopRegionDiv.style.cssText = `
      position: absolute;
      top: 0;
      bottom: 0;
      background: rgba(74, 158, 255, 0.2);
      left: 0;
      width: 100%;
      display: ${this.loopEnabled ? "block" : "none"};
      pointer-events: none;
    `;
    timeline.appendChild(this.loopRegionDiv);

    // Loop start handle
    this.loopStartHandle = document.createElement("div");
    this.loopStartHandle.style.cssText = `
      position: absolute;
      top: 0;
      bottom: 0;
      width: 8px;
      background: #4a9eff;
      left: 0;
      cursor: ew-resize;
      display: ${this.loopEnabled ? "block" : "none"};
    `;
    this.setupLoopHandleDrag(this.loopStartHandle, "start");
    timeline.appendChild(this.loopStartHandle);

    // Loop end handle
    this.loopEndHandle = document.createElement("div");
    this.loopEndHandle.style.cssText = `
      position: absolute;
      top: 0;
      bottom: 0;
      width: 8px;
      background: #4a9eff;
      right: 0;
      cursor: ew-resize;
      display: ${this.loopEnabled ? "block" : "none"};
    `;
    this.setupLoopHandleDrag(this.loopEndHandle, "end");
    timeline.appendChild(this.loopEndHandle);

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

    // Click to seek (only if not clicking on handles)
    timeline.addEventListener("click", (e) => {
      if (e.target === this.loopStartHandle || e.target === this.loopEndHandle) return;
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

    // Mode toggle group
    const modeGroup = document.createElement("div");
    modeGroup.style.cssText = "display: flex; gap: 2px; margin-left: 8px;";

    const activeModeStyle = `
      padding: 4px 8px;
      background: #4a9eff;
      color: #fff;
      border: 1px solid #4a9eff;
      border-radius: 4px 0 0 4px;
      cursor: pointer;
      font-size: 10px;
    `;
    const inactiveModeStyle = `
      padding: 4px 8px;
      background: #333;
      color: #888;
      border: 1px solid #444;
      border-radius: 0 4px 4px 0;
      cursor: pointer;
      font-size: 10px;
    `;

    this.specModeBtn = document.createElement("button");
    this.specModeBtn.textContent = "Spec";
    this.specModeBtn.style.cssText = this.previewMode === "spec" ? activeModeStyle : inactiveModeStyle.replace("0 4px 4px 0", "4px 0 0 4px");
    this.specModeBtn.onclick = () => this.setPreviewMode("spec");

    this.exportedModeBtn = document.createElement("button");
    this.exportedModeBtn.textContent = "Export";
    this.exportedModeBtn.style.cssText = this.previewMode === "exported" ? activeModeStyle.replace("4px 0 0 4px", "0 4px 4px 0") : inactiveModeStyle;
    this.exportedModeBtn.onclick = () => this.setPreviewMode("exported");

    modeGroup.appendChild(this.specModeBtn);
    modeGroup.appendChild(this.exportedModeBtn);
    transport.appendChild(modeGroup);

    // Render mode selector
    const renderGroup = document.createElement("div");
    renderGroup.style.cssText = "display: flex; align-items: center; gap: 4px; margin-left: 8px;";
    const renderLabel = document.createElement("span");
    renderLabel.textContent = "Render:";
    renderLabel.style.cssText = "font-size: 10px; color: #888;";

    this.renderModeSelect = document.createElement("select");
    this.renderModeSelect.style.cssText = `
      padding: 2px 6px;
      background: #333;
      color: #ccc;
      border: 1px solid #444;
      border-radius: 3px;
      font-size: 10px;
    `;
    this.renderModeSelect.innerHTML = `
      <option value="lit">Lit</option>
      <option value="unlit">Unlit</option>
      <option value="wireframe">Wireframe</option>
    `;
    this.renderModeSelect.value = this.renderMode;
    this.renderModeSelect.addEventListener("change", () => {
      this.setRenderMode(this.renderModeSelect.value as RenderMode);
    });

    renderGroup.appendChild(renderLabel);
    renderGroup.appendChild(this.renderModeSelect);
    transport.appendChild(renderGroup);

    // Camera preset buttons
    const cameraGroup = document.createElement("div");
    cameraGroup.style.cssText = "display: flex; gap: 2px; margin-left: 8px;";

    const cameraPresets: Array<{ label: string; preset: CameraPreset }> = [
      { label: "3/4", preset: "three-quarter" },
      { label: "F", preset: "front" },
      { label: "S", preset: "side" },
      { label: "B", preset: "back" },
      { label: "T", preset: "top" },
    ];

    const smallBtnStyle = `
      padding: 2px 6px;
      background: #333;
      color: #888;
      border: 1px solid #444;
      border-radius: 3px;
      cursor: pointer;
      font-size: 9px;
    `;

    for (const { label, preset } of cameraPresets) {
      const btn = document.createElement("button");
      btn.textContent = label;
      btn.title = preset.charAt(0).toUpperCase() + preset.slice(1).replace("-", " ") + " view";
      btn.style.cssText = smallBtnStyle;
      btn.onclick = () => this.setCameraPreset(preset);
      cameraGroup.appendChild(btn);
    }
    transport.appendChild(cameraGroup);

    // Loop toggle
    this.loopToggleBtn = document.createElement("button");
    this.loopToggleBtn.textContent = "Loop";
    this.loopToggleBtn.style.cssText = buttonStyle + (this.loopEnabled ? " background: #4a9eff; color: #fff;" : "");
    this.loopToggleBtn.onclick = () => this.toggleLoop();
    transport.appendChild(this.loopToggleBtn);

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

  /**
   * Set up drag behavior for loop region handles.
   */
  private setupLoopHandleDrag(handle: HTMLDivElement, which: "start" | "end"): void {
    let dragging = false;

    const onMouseMove = (e: MouseEvent) => {
      if (!dragging) return;
      const timeline = this.timelineDiv;
      const rect = timeline.getBoundingClientRect();
      const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
      const progress = x / rect.width;

      if (which === "start") {
        // Clamp start to be before end
        this.loopStart = Math.min(progress, this.loopEnd - 0.01);
      } else {
        // Clamp end to be after start
        this.loopEnd = Math.max(progress, this.loopStart + 0.01);
      }

      this.updateLoopRegionUI();
      this.saveSettings();
    };

    const onMouseUp = () => {
      dragging = false;
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    };

    handle.addEventListener("mousedown", (e) => {
      e.stopPropagation();
      dragging = true;
      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    });
  }

  /**
   * Update loop region UI positions based on loopStart/loopEnd.
   */
  private updateLoopRegionUI(): void {
    const startPercent = this.loopStart * 100;
    const endPercent = this.loopEnd * 100;

    this.loopRegionDiv.style.left = `${startPercent}%`;
    this.loopRegionDiv.style.width = `${endPercent - startPercent}%`;

    this.loopStartHandle.style.left = `${startPercent}%`;
    this.loopEndHandle.style.left = `${endPercent}%`;
    this.loopEndHandle.style.right = "auto";
  }

  /**
   * Toggle loop playback on/off.
   */
  private toggleLoop(): void {
    this.loopEnabled = !this.loopEnabled;
    this.updateLoopVisibility();
    this.saveSettings();

    // Update action loop mode
    if (this.currentAction) {
      this.currentAction.loop = this.loopEnabled ? THREE.LoopRepeat : THREE.LoopOnce;
    }
  }

  /**
   * Render keyframe markers on the timeline.
   */
  private renderKeyframeMarkers(): void {
    // Clear existing markers
    this.keyframeMarkers.forEach((marker) => marker.remove());
    this.keyframeMarkers = [];

    // Get animation duration
    const duration = this.currentGltf?.animations[0]?.duration;
    if (!duration || duration <= 0) return;

    // Get keyframe times from metadata
    const times = this.metadata?.keyframe_times;
    if (!times || times.length === 0) return;

    times.forEach((time) => {
      const progress = time / duration;
      if (progress < 0 || progress > 1) return;

      const marker = document.createElement("div");
      marker.style.cssText = `
        position: absolute;
        top: 4px;
        bottom: 4px;
        width: 2px;
        background: rgba(255, 200, 100, 0.6);
        left: ${progress * 100}%;
        transform: translateX(-50%);
        pointer-events: auto;
        cursor: pointer;
        border-radius: 1px;
      `;
      marker.title = `Keyframe at ${this.formatTime(time)}`;
      marker.addEventListener("click", (e) => {
        e.stopPropagation();
        this.seekToTime(time);
      });
      this.keyframeMarkersContainer.appendChild(marker);
      this.keyframeMarkers.push(marker);
    });
  }

  /**
   * Seek to a specific time in seconds.
   */
  private seekToTime(time: number): void {
    if (!this.currentAction || !this.currentGltf?.animations[0]) return;
    const duration = this.currentGltf.animations[0].duration;
    this.currentAction.time = Math.max(0, Math.min(time, duration));
    this.mixer?.update(0);
    this.updateTimelineUI();
  }

  /**
   * Update visibility of loop UI elements.
   */
  private updateLoopVisibility(): void {
    const display = this.loopEnabled ? "block" : "none";
    this.loopRegionDiv.style.display = display;
    this.loopStartHandle.style.display = display;
    this.loopEndHandle.style.display = display;

    if (this.loopToggleBtn) {
      this.loopToggleBtn.style.background = this.loopEnabled ? "#4a9eff" : "#333";
      this.loopToggleBtn.style.color = this.loopEnabled ? "#fff" : "#888";
    }
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

    // Bone list container (left) - includes filter input and tree
    const boneListContainer = document.createElement("div");
    boneListContainer.style.cssText = `
      width: 200px;
      display: flex;
      flex-direction: column;
      border-right: 1px solid #333;
    `;

    // Filter input
    const filterInput = document.createElement("input");
    filterInput.type = "text";
    filterInput.placeholder = "Filter bones...";
    filterInput.style.cssText = `
      padding: 4px 8px;
      background: #252525;
      border: none;
      border-bottom: 1px solid #333;
      color: #ccc;
      font-size: 11px;
      outline: none;
    `;
    filterInput.addEventListener("input", () => {
      this.boneFilter = filterInput.value.toLowerCase();
      this.rebuildBoneTree();
    });
    boneListContainer.appendChild(filterInput);

    // Bone tree container
    this.boneList = document.createElement("div");
    this.boneList.style.cssText = `
      flex: 1;
      overflow-y: auto;
      font-size: 11px;
    `;
    boneListContainer.appendChild(this.boneList);
    panel.appendChild(boneListContainer);

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

          // Cache original materials
          this.originalMaterials.clear();
          this.disposeOverrideMaterials();
          gltf.scene.traverse((child) => {
            if ((child as THREE.Mesh).isMesh) {
              const mesh = child as THREE.Mesh;
              this.originalMaterials.set(mesh, mesh.material);
            }
          });

          // Auto-fit camera
          this.fitCameraToModel(gltf.scene);

          // Build bone list
          this.buildBoneList(gltf.scene);

          // Update bone visibility
          this.updateBoneVisibility();

          // Apply current render mode
          this.applyMaterials();

          // Update info display
          this.updateInfoDisplay();

          // Render keyframe markers
          this.renderKeyframeMarkers();

          // Hide generating status
          this.hideGenerating();

          resolve();
        },
        (error) => {
          this.hideGenerating();
          reject(new Error(`Failed to parse GLB: ${error}`));
        }
      );
    });
  }

  private fitCameraToModel(_model: THREE.Object3D): void {
    this.setCameraPreset("three-quarter");
  }

  /**
   * Set camera to a preset view angle.
   */
  private setCameraPreset(preset: CameraPreset): void {
    if (!this.currentModel) return;

    const box = new THREE.Box3().setFromObject(this.currentModel);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());

    const maxDim = Math.max(size.x, size.y, size.z);
    const fov = this.camera.fov * (Math.PI / 180);
    const distance = (maxDim / (2 * Math.tan(fov / 2))) * 1.5;

    let position: THREE.Vector3;

    switch (preset) {
      case "front":
        position = new THREE.Vector3(center.x, center.y, center.z + distance);
        break;
      case "back":
        position = new THREE.Vector3(center.x, center.y, center.z - distance);
        break;
      case "side":
        position = new THREE.Vector3(center.x + distance, center.y, center.z);
        break;
      case "top":
        position = new THREE.Vector3(center.x, center.y + distance, center.z);
        break;
      case "three-quarter":
      default:
        position = new THREE.Vector3(
          center.x + distance * 0.7,
          center.y + distance * 0.5,
          center.z + distance * 0.7
        );
        break;
    }

    this.camera.position.copy(position);
    this.controls.target.copy(center);
    this.controls.update();
  }

  private buildBoneList(scene: THREE.Object3D): void {
    this.bones = [];

    scene.traverse((obj) => {
      if ((obj as THREE.Bone).isBone) {
        this.bones.push(obj as THREE.Bone);
      }
    });

    // By default, expand root bones
    this.expandedBones.clear();
    for (const bone of this.bones) {
      if (!bone.parent || !(bone.parent as THREE.Bone).isBone) {
        this.expandedBones.add(this.getBoneId(bone));
      }
    }

    this.rebuildBoneTree();
  }

  /**
   * Get a unique identifier for a bone.
   */
  private getBoneId(bone: THREE.Bone): string {
    return bone.uuid;
  }

  /**
   * Rebuild the bone tree UI with current filter and expansion state.
   */
  private rebuildBoneTree(): void {
    this.boneList.innerHTML = "";

    // Find root bones (bones whose parent is not a bone)
    const rootBones = this.bones.filter(
      (bone) => !bone.parent || !(bone.parent as THREE.Bone).isBone
    );

    for (const rootBone of rootBones) {
      this.renderBoneTreeNode(rootBone, 0);
    }
  }

  /**
   * Render a bone tree node and its children recursively.
   */
  private renderBoneTreeNode(bone: THREE.Bone, depth: number): void {
    const boneName = bone.name || "Unnamed";
    const boneId = this.getBoneId(bone);

    // Filter check - if filter is set, only show matching bones and their ancestors/descendants
    const matchesFilter = this.boneFilter === "" || boneName.toLowerCase().includes(this.boneFilter);

    // Get children bones
    const childBones = bone.children.filter((child) => (child as THREE.Bone).isBone) as THREE.Bone[];
    const hasChildren = childBones.length > 0;

    // Check if any descendant matches filter
    const hasMatchingDescendant = this.boneFilter !== "" && this.hasMatchingDescendant(bone);

    // Skip if no match and no matching descendants
    if (this.boneFilter !== "" && !matchesFilter && !hasMatchingDescendant) {
      return;
    }

    const isExpanded = this.expandedBones.has(boneId);
    const isSelected = this.selectedBone === bone;

    // Create item container
    const item = document.createElement("div");
    item.style.cssText = `
      padding: 2px 8px 2px ${8 + depth * 12}px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 4px;
      ${isSelected ? "background: #4a9eff; color: #fff;" : ""}
    `;
    item.dataset.boneId = boneId;

    // Expand/collapse toggle
    if (hasChildren) {
      const toggle = document.createElement("span");
      toggle.textContent = isExpanded ? "▼" : "▶";
      toggle.style.cssText = `
        font-size: 8px;
        width: 10px;
        user-select: none;
      `;
      toggle.onclick = (e) => {
        e.stopPropagation();
        if (isExpanded) {
          this.expandedBones.delete(boneId);
        } else {
          this.expandedBones.add(boneId);
        }
        this.rebuildBoneTree();
      };
      item.appendChild(toggle);
    } else {
      // Spacer for alignment
      const spacer = document.createElement("span");
      spacer.style.width = "10px";
      item.appendChild(spacer);
    }

    // Bone name
    const nameSpan = document.createElement("span");
    nameSpan.textContent = boneName;
    if (this.boneFilter && matchesFilter) {
      nameSpan.style.fontWeight = "bold";
    }
    item.appendChild(nameSpan);

    item.onclick = () => {
      this.selectBone(bone);
      this.highlightBoneInList(bone);
    };

    this.boneList.appendChild(item);

    // Render children if expanded
    if (hasChildren && (isExpanded || (this.boneFilter !== "" && hasMatchingDescendant))) {
      for (const childBone of childBones) {
        this.renderBoneTreeNode(childBone, depth + 1);
      }
    }
  }

  /**
   * Check if a bone or any of its descendants matches the current filter.
   */
  private hasMatchingDescendant(bone: THREE.Bone): boolean {
    for (const child of bone.children) {
      if ((child as THREE.Bone).isBone) {
        const childBone = child as THREE.Bone;
        const childName = childBone.name || "Unnamed";
        if (childName.toLowerCase().includes(this.boneFilter)) {
          return true;
        }
        if (this.hasMatchingDescendant(childBone)) {
          return true;
        }
      }
    }
    return false;
  }

  /**
   * Handle click on viewport to select bone via raycasting.
   */
  private handleViewportClick(event: MouseEvent): void {
    if (!this.currentModel || this.bones.length === 0) return;

    // Get mouse coordinates in normalized device coordinates (-1 to +1)
    const rect = this.renderer.domElement.getBoundingClientRect();
    const x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    const y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

    // Set up raycaster
    this.raycaster.setFromCamera(new THREE.Vector2(x, y), this.camera);

    // Find all meshes in the model
    const meshes: THREE.Mesh[] = [];
    this.currentModel.traverse((obj) => {
      if ((obj as THREE.Mesh).isMesh) {
        meshes.push(obj as THREE.Mesh);
      }
    });

    // Test intersection with meshes
    const intersects = this.raycaster.intersectObjects(meshes, false);
    if (intersects.length === 0) return;

    // Get the intersection point
    const hitPoint = intersects[0].point;

    // Find the nearest bone to the hit point
    let nearestBone: THREE.Bone | null = null;
    let nearestDistance = Infinity;

    for (const bone of this.bones) {
      const boneWorldPos = new THREE.Vector3();
      bone.getWorldPosition(boneWorldPos);
      const distance = hitPoint.distanceTo(boneWorldPos);
      if (distance < nearestDistance) {
        nearestDistance = distance;
        nearestBone = bone;
      }
    }

    if (nearestBone) {
      this.selectBone(nearestBone);
      this.highlightBoneInList(nearestBone);
    }
  }

  /**
   * Highlight the selected bone in the bone list panel.
   */
  private highlightBoneInList(bone: THREE.Bone): void {
    const boneId = this.getBoneId(bone);

    // Expand ancestors to make sure the bone is visible
    let parent = bone.parent;
    while (parent) {
      if ((parent as THREE.Bone).isBone) {
        this.expandedBones.add(this.getBoneId(parent as THREE.Bone));
      }
      parent = parent.parent;
    }

    // Rebuild to ensure the bone is visible
    this.rebuildBoneTree();

    // Find and scroll to the item
    const items = this.boneList.querySelectorAll("[data-bone-id]");
    for (const item of items) {
      if ((item as HTMLDivElement).dataset.boneId === boneId) {
        item.scrollIntoView({ behavior: "smooth", block: "nearest" });
        break;
      }
    }
  }

  private selectBone(bone: THREE.Bone): void {
    this.selectedBone = bone;
    this.updateSelectedBoneHighlight();
    this.updateBoneInspector();
  }

  /**
   * Update the 3D highlight for selected bone.
   */
  private updateSelectedBoneHighlight(): void {
    // Remove previous highlight
    if (this.selectedBoneHelper) {
      this.selectedBoneHelper.parent?.remove(this.selectedBoneHelper);
      this.selectedBoneHelper = null;
    }

    // Add new highlight for selected bone
    if (this.selectedBone) {
      this.selectedBoneHelper = new THREE.AxesHelper(0.15);
      // Make the helper more visible (thicker lines via scale)
      this.selectedBoneHelper.scale.set(1.5, 1.5, 1.5);
      this.selectedBone.add(this.selectedBoneHelper);
    }
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

  // Mode controls
  setModeChangeCallback(callback: ((mode: PreviewMode) => void) | null): void {
    this.onModeChange = callback;
  }

  /**
   * Show the "Generating..." overlay.
   * Call this when starting a new preview generation.
   */
  showGenerating(): void {
    this.statusOverlay.style.display = "flex";
  }

  /**
   * Hide the "Generating..." overlay.
   * Called automatically when GLB loads.
   */
  hideGenerating(): void {
    this.statusOverlay.style.display = "none";
  }

  setPreviewMode(mode: PreviewMode): void {
    if (this.previewMode === mode) return;
    this.previewMode = mode;
    this.updateModeButtonStyles();
    this.saveSettings();
    if (this.onModeChange) {
      this.onModeChange(mode);
    }
  }

  getPreviewMode(): PreviewMode {
    return this.previewMode;
  }

  private updateModeButtonStyles(): void {
    const activeStyle = "background: #4a9eff; color: #fff; border-color: #4a9eff;";
    const inactiveStyle = "background: #333; color: #888; border-color: #444;";

    if (this.specModeBtn && this.exportedModeBtn) {
      if (this.previewMode === "spec") {
        this.specModeBtn.style.cssText = this.specModeBtn.style.cssText.replace(/background:[^;]+;/, activeStyle.split(";")[0] + ";").replace(/color:[^;]+;/, activeStyle.split(";")[1] + ";").replace(/border-color:[^;]+;/, activeStyle.split(";")[2] + ";");
        this.exportedModeBtn.style.cssText = this.exportedModeBtn.style.cssText.replace(/background:[^;]+;/, inactiveStyle.split(";")[0] + ";").replace(/color:[^;]+;/, inactiveStyle.split(";")[1] + ";").replace(/border-color:[^;]+;/, inactiveStyle.split(";")[2] + ";");
      } else {
        this.specModeBtn.style.cssText = this.specModeBtn.style.cssText.replace(/background:[^;]+;/, inactiveStyle.split(";")[0] + ";").replace(/color:[^;]+;/, inactiveStyle.split(";")[1] + ";").replace(/border-color:[^;]+;/, inactiveStyle.split(";")[2] + ";");
        this.exportedModeBtn.style.cssText = this.exportedModeBtn.style.cssText.replace(/background:[^;]+;/, activeStyle.split(";")[0] + ";").replace(/color:[^;]+;/, activeStyle.split(";")[1] + ";").replace(/border-color:[^;]+;/, activeStyle.split(";")[2] + ";");
      }
    }
  }

  // Render mode controls
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

  getRenderMode(): RenderMode {
    return this.renderMode;
  }

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
      // Apply MeshBasicMaterial
      this.originalMaterials.forEach((_, mesh) => {
        const mat = new THREE.MeshBasicMaterial({
          color: 0x888888,
        });
        this.overrideMaterials.push(mat);
        mesh.material = mat;
      });
    } else if (this.renderMode === "wireframe") {
      // Apply wireframe material
      this.originalMaterials.forEach((_, mesh) => {
        const mat = new THREE.MeshBasicMaterial({
          color: 0x00ff00,
          wireframe: true,
        });
        this.overrideMaterials.push(mat);
        mesh.material = mat;
      });
    }
  }

  private disposeOverrideMaterials(): void {
    for (const mat of this.overrideMaterials) {
      mat.dispose();
    }
    this.overrideMaterials = [];
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

      // Loop region clamping
      if (this.loopEnabled && this.currentAction && this.currentGltf?.animations[0]) {
        const duration = this.currentGltf.animations[0].duration;
        const loopStartTime = this.loopStart * duration;
        const loopEndTime = this.loopEnd * duration;

        if (this.currentAction.time >= loopEndTime) {
          this.currentAction.time = loopStartTime;
        } else if (this.currentAction.time < loopStartTime) {
          this.currentAction.time = loopStartTime;
        }
      }

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
    this.clearSelectedBoneHelper();
    this.bones = [];
    this.selectedBone = null;
    this.originalMaterials.clear();
    this.disposeOverrideMaterials();
    this.boneList.innerHTML = "";
    this.boneInspector.textContent = "Select a bone to inspect";
    this.infoDiv.textContent = "No animation loaded";
  }

  private clearSelectedBoneHelper(): void {
    if (this.selectedBoneHelper) {
      this.selectedBoneHelper.parent?.remove(this.selectedBoneHelper);
      this.selectedBoneHelper = null;
    }
  }

  dispose(): void {
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
    }
    this.controls.dispose();
    this.renderer.dispose();
    this.clearBoneHelpers();
    this.clearSelectedBoneHelper();
    this.bones = [];
    this.originalMaterials.clear();
    this.disposeOverrideMaterials();
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
