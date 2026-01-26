import { loadInspectSection, saveInspectSection } from "../lib/storage";
import { clamp01, wipeToSplitX } from "../lib/compare";

export type PreviewQuality = "proxy" | "full";
export type TexturePreset = "pixel" | "material";
export type TextureSampling = "nearest" | "linear";
export type TextureChannel = "RGB" | "R" | "G" | "B" | "A";

export type TextureInspectorStateV1 = {
  v: 1;
  preset: TexturePreset;
  sampling: TextureSampling;
  channel: TextureChannel;
  tiling: boolean;
  seam: {
    enabled: boolean;
    sensitivity: number; // 0..255
  };
  mipLevel: number;
};

type TexelCoord = {
  x: number;
  y: number;
  mip: number;
};

type SeamCache = {
  width: number;
  height: number;
  rowMaxDelta: Uint8Array;
  colMaxDelta: Uint8Array;
};

type CompareSnapshot = {
  sourceLevel: number;
  channel: TextureChannel;
  canvas: HTMLCanvasElement;
};

/**
 * Texture preview component with zoom, pan, tiling, and basic inspection.
 */
export class TexturePreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private overlayCanvas: HTMLCanvasElement;
  private overlayCtx: CanvasRenderingContext2D;
  private controlsDiv: HTMLDivElement;
  private infoDiv: HTMLDivElement;
  private pixelInfoDiv: HTMLDivElement;
  private tileBtn: HTMLButtonElement;
  private seamBtn: HTMLButtonElement;
  private compareBtn: HTMLButtonElement;
  private snapBtn: HTMLButtonElement;
  private wipeSlider: HTMLInputElement;
  private presetSelect: HTMLSelectElement;
  private samplingSelect: HTMLSelectElement;
  private channelSelect: HTMLSelectElement;
  private mipSelect: HTMLSelectElement;
  private seamSensitivitySlider: HTMLInputElement;
  private resizeObserver: ResizeObserver;

  private wheelListenerOptions: AddEventListenerOptions = { passive: false };

  // Bound event handlers for cleanup
  private boundHandleWheel: (e: WheelEvent) => void;
  private boundHandleMouseDown: (e: MouseEvent) => void;
  private boundHandleMouseMove: (e: MouseEvent) => void;
  private boundHandleMouseUp: () => void;

  private image: HTMLImageElement | null = null;
  private lastMetadata: Record<string, unknown> | undefined;
  private currentMimeType: string | null = null;
  private currentQuality: PreviewQuality = "proxy";
  private filePathForState: string = "editor.star";
  private assetTypeForPreset: string | null = null;

  private preset: TexturePreset = "material";
  private sampling: TextureSampling = "nearest";
  private zoom = 1;
  private panX = 0;
  private panY = 0;
  private tiling = false;
  private seam = false;
  private seamSensitivity = 32;
  private channel: TextureChannel = "RGB";
  private mipLevel = 0;
  private maxMipLevel = 0;

  private compareEnabled = false;
  private compareWipe = 0.5;
  private compareSnapshot: CompareSnapshot | null = null;

  private isPointerDown = false;
  private pointerDownClientX = 0;
  private pointerDownClientY = 0;
  private isPanning = false;
  private lastMouseX = 0;
  private lastMouseY = 0;

  private hoverTexel: TexelCoord | null = null;
  private pinnedTexel: TexelCoord | null = null;
  private pendingOverlayRaf: number | null = null;

  private dpr = 1;
  private canvasCssWidth = 0;
  private canvasCssHeight = 0;

  private mipCanvasCache = new Map<number, HTMLCanvasElement>();
  private filteredCanvasCache = new Map<string, HTMLCanvasElement>();
  private mipImageDataCache = new Map<number, ImageData>();
  private seamCache = new Map<number, SeamCache>();

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

    // Info bar
    this.infoDiv = document.createElement("div");
    this.infoDiv.style.cssText = `
      font-size: 11px;
      color: #999;
      display: flex;
      gap: 12px;
      flex-wrap: wrap;
    `;
    this.infoDiv.innerHTML = `<span>No texture loaded</span>`;
    wrapper.appendChild(this.infoDiv);

    // Pixel inspector readout
    this.pixelInfoDiv = document.createElement("div");
    this.pixelInfoDiv.style.cssText = `
      font-size: 11px;
      color: #bdbdbd;
      font-family: Consolas, Monaco, monospace;
      user-select: text;
      min-height: 16px;
    `;
    this.pixelInfoDiv.textContent = "";
    wrapper.appendChild(this.pixelInfoDiv);

    // Canvas area (base + overlay)
    const canvasWrap = document.createElement("div");
    canvasWrap.style.cssText = `
      position: relative;
      flex: 1;
      width: 100%;
      background: #1e1e1e;
      border-radius: 4px;
      overflow: hidden;
    `;

    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = `
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      cursor: grab;
    `;
    canvasWrap.appendChild(this.canvas);
    this.ctx = this.canvas.getContext("2d")!;

    this.overlayCanvas = document.createElement("canvas");
    this.overlayCanvas.style.cssText = `
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      pointer-events: none;
    `;
    canvasWrap.appendChild(this.overlayCanvas);
    this.overlayCtx = this.overlayCanvas.getContext("2d")!;

    wrapper.appendChild(canvasWrap);

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
    this.tileBtn = this.createButton("Tile", () => this.toggleTiling());
    this.seamBtn = this.createButton("Seam", () => this.toggleSeam());

    this.compareBtn = this.createButton("Compare", () => this.toggleCompare());
    this.snapBtn = this.createButton("Snap A", () => this.snapCompareA());
    this.wipeSlider = document.createElement("input");
    this.wipeSlider.type = "range";
    this.wipeSlider.min = "0";
    this.wipeSlider.max = "1";
    this.wipeSlider.step = "0.001";
    this.wipeSlider.value = String(this.compareWipe);
    this.wipeSlider.style.cssText = `
      width: 140px;
      display: none;
    `;
    this.wipeSlider.addEventListener("input", () => {
      const n = Number(this.wipeSlider.value);
      this.compareWipe = clamp01(n);
      this.draw();
    });

    this.presetSelect = this.createSelect(["Pixel", "Material"], (value) => {
      const next = value === "Pixel" ? "pixel" : "material";
      this.preset = next;
      // Preset defaults.
      this.sampling = next === "pixel" ? "nearest" : "linear";
      this.samplingSelect.value = this.sampling === "nearest" ? "Nearest" : "Linear";
      this.persistInspectorState();
      this.draw();
    });

    this.samplingSelect = this.createSelect(["Nearest", "Linear"], (value) => {
      this.sampling = value === "Linear" ? "linear" : "nearest";
      this.persistInspectorState();
      this.draw();
    });

    this.channelSelect = this.createSelect(["RGB", "R", "G", "B", "A"], (value) => {
      this.channel = value as typeof this.channel;
      this.persistInspectorState();
      this.draw();
    });

    this.mipSelect = this.createSelect(["Mip 0"], (value) => {
      const m = Number.parseInt(value.replace(/^Mip\s+/, ""), 10);
      this.mipLevel = Number.isFinite(m) ? Math.max(0, Math.min(this.maxMipLevel, m)) : 0;
      this.persistInspectorState();
      this.draw();
    });

    this.seamSensitivitySlider = document.createElement("input");
    this.seamSensitivitySlider.type = "range";
    this.seamSensitivitySlider.min = "0";
    this.seamSensitivitySlider.max = "255";
    this.seamSensitivitySlider.step = "1";
    this.seamSensitivitySlider.value = String(this.seamSensitivity);
    this.seamSensitivitySlider.style.cssText = `
      width: 120px;
      display: none;
    `;
    this.seamSensitivitySlider.addEventListener("input", () => {
      const n = Number(this.seamSensitivitySlider.value);
      this.seamSensitivity = Number.isFinite(n) ? Math.max(0, Math.min(255, Math.round(n))) : 32;
      this.persistInspectorState();
      this.draw();
    });

    this.controlsDiv.appendChild(zoomOutBtn);
    this.controlsDiv.appendChild(zoomInBtn);
    this.controlsDiv.appendChild(resetBtn);
    this.controlsDiv.appendChild(this.tileBtn);
    this.controlsDiv.appendChild(this.seamBtn);
    this.controlsDiv.appendChild(this.compareBtn);
    this.controlsDiv.appendChild(this.snapBtn);
    this.controlsDiv.appendChild(this.wipeSlider);
    this.controlsDiv.appendChild(this.seamSensitivitySlider);
    this.controlsDiv.appendChild(this.presetSelect);
    this.controlsDiv.appendChild(this.samplingSelect);
    this.controlsDiv.appendChild(this.channelSelect);
    this.controlsDiv.appendChild(this.mipSelect);

    wrapper.appendChild(this.controlsDiv);
    container.appendChild(wrapper);

    // Bind event handlers for proper cleanup
    this.boundHandleWheel = (e) => this.handleWheel(e);
    this.boundHandleMouseDown = (e) => this.handleMouseDown(e);
    this.boundHandleMouseMove = (e) => this.handleMouseMove(e);
    this.boundHandleMouseUp = () => this.handleMouseUp();

    // Set up event listeners
    this.canvas.addEventListener("wheel", this.boundHandleWheel, this.wheelListenerOptions);
    this.canvas.addEventListener("mousedown", this.boundHandleMouseDown);
    this.canvas.addEventListener("mousemove", this.boundHandleMouseMove);
    this.canvas.addEventListener("mouseup", this.boundHandleMouseUp);
    this.canvas.addEventListener("mouseleave", this.boundHandleMouseUp);

    // Handle resize
    this.resizeObserver = new ResizeObserver(() => this.handleResize());
    this.resizeObserver.observe(container);
    this.handleResize();

    this.drawEmpty();
    this.updateCompareControls();
  }

  private setButtonEnabled(btn: HTMLButtonElement, enabled: boolean): void {
    btn.disabled = !enabled;
    btn.style.opacity = enabled ? "1" : "0.5";
    btn.style.cursor = enabled ? "pointer" : "not-allowed";
  }

  private updateCompareControls(): void {
    this.compareBtn.style.background = this.compareEnabled ? "#007acc" : "#444";
    const snapEnabled = this.compareEnabled && this.image !== null && !this.seam;
    this.setButtonEnabled(this.snapBtn, snapEnabled);
    this.wipeSlider.style.display = this.compareEnabled && !this.seam ? "block" : "none";
    this.wipeSlider.value = String(this.compareWipe);
  }

  private toggleCompare(): void {
    const next = !this.compareEnabled;
    if (next && this.seam) {
      this.seam = false;
      this.seamBtn.style.background = "#444";
      this.seamSensitivitySlider.style.display = "none";
      this.persistInspectorState();
    }

    this.compareEnabled = next;
    this.updateCompareControls();
    this.draw();
  }

  private snapCompareA(): void {
    if (!this.compareEnabled || !this.image || this.seam) return;

    const source =
      this.channel === "RGB"
        ? this.getMipCanvas(this.mipLevel)
        : this.getFilteredCanvas(this.mipLevel, this.channel);

    const snap = document.createElement("canvas");
    snap.width = source.width;
    snap.height = source.height;
    const snapCtx = snap.getContext("2d", { willReadFrequently: true });
    if (!snapCtx) return;
    snapCtx.imageSmoothingEnabled = false;
    snapCtx.drawImage(source, 0, 0);

    this.compareSnapshot = {
      sourceLevel: this.mipLevel,
      channel: this.channel,
      canvas: snap,
    };

    this.draw();
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

  private createSelect(options: string[], onChange: (value: string) => void): HTMLSelectElement {
    const select = document.createElement("select");
    select.style.cssText = `
      padding: 4px 10px;
      background: #333;
      color: white;
      border: 1px solid #555;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
      height: 26px;
    `;

    for (const optionText of options) {
      const opt = document.createElement("option");
      opt.value = optionText;
      opt.textContent = optionText;
      select.appendChild(opt);
    }

    select.onchange = () => onChange(select.value);
    return select;
  }

  /**
   * Load texture from base64 data.
   */
  loadTexture(
    base64Data: string,
    mimeType: string,
    metadata?: Record<string, unknown>,
    opts?: {
      filePath?: string;
      assetType?: string;
      quality?: PreviewQuality;
    }
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        this.image = img;
        this.lastMetadata = metadata;
        this.currentMimeType = mimeType;
        this.filePathForState = opts?.filePath ?? this.filePathForState;
        this.assetTypeForPreset = opts?.assetType ?? this.assetTypeForPreset;
        this.currentQuality = opts?.quality ?? this.currentQuality;

        this.compareSnapshot = null;
        this.updateCompareControls();

        this.clearDerivedCaches();
        this.maxMipLevel = this.computeMaxMipLevel(img.width, img.height);

        this.applyInspectorStateForCurrentFile();
        this.resetView();
        this.updateInfo();
        this.draw();
        resolve();
      };
      img.onerror = () => reject(new Error("Failed to load texture"));
      img.src = `data:${mimeType};base64,${base64Data}`;
    });
  }

  private updateInfo(): void {
    if (!this.image) {
      this.infoDiv.innerHTML = `<span>No texture loaded</span>`;
      return;
    }

    const parts = [
      `<span>${this.image.width}×${this.image.height}</span>`,
      `<span>Zoom: ${Math.round(this.zoom * 100)}%</span>`,
      `<span>Mip: ${this.mipLevel}</span>`,
      `<span>${this.channel}</span>`,
      `<span>${this.sampling === "linear" ? "Linear" : "Nearest"}</span>`,
    ];

    const md = this.lastMetadata;
    if (md) {
      if ((md as any).tileable) {
        parts.push(`<span style="color: #4a9eff;">Tileable</span>`);
      }
    }

    if (this.currentMimeType) {
      parts.push(`<span style="color: #888;">${this.currentMimeType}</span>`);
    }

    parts.push(
      this.currentQuality === "full"
        ? `<span style="color: #ddd;">Full</span>`
        : `<span style="color: #888;">Preview</span>`
    );

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
    this.tileBtn.style.background = this.tiling ? "#007acc" : "#444";
    this.persistInspectorState();
    this.draw();
  }

  private toggleSeam(): void {
    this.seam = !this.seam;
    this.seamBtn.style.background = this.seam ? "#007acc" : "#444";
    this.seamSensitivitySlider.style.display = this.seam ? "block" : "none";

    if (this.seam) {
      this.compareEnabled = false;
      this.updateCompareControls();
    }

    this.persistInspectorState();
    this.draw();
  }

  private populateMipSelect(maxLevel: number): void {
    this.mipSelect.innerHTML = "";
    for (let i = 0; i <= maxLevel; i++) {
      const opt = document.createElement("option");
      opt.value = `Mip ${i}`;
      opt.textContent = `Mip ${i}`;
      this.mipSelect.appendChild(opt);
    }
    this.mipSelect.value = `Mip ${this.mipLevel}`;
  }

  private computeMaxMipLevel(width: number, height: number): number {
    let w = Math.max(1, Math.floor(width));
    let h = Math.max(1, Math.floor(height));
    let level = 0;
    while (w > 1 || h > 1) {
      w = Math.max(1, Math.floor(w / 2));
      h = Math.max(1, Math.floor(h / 2));
      level++;
    }
    return level;
  }

  private clearDerivedCaches(): void {
    this.mipCanvasCache.clear();
    this.filteredCanvasCache.clear();
    this.mipImageDataCache.clear();
    this.seamCache.clear();
  }

  private applyInspectorStateForCurrentFile(): void {
    const defaultPreset = this.defaultPresetForAssetType(this.assetTypeForPreset);
    const fallback: TextureInspectorStateV1 = {
      v: 1,
      preset: defaultPreset,
      sampling: defaultPreset === "pixel" ? "nearest" : "linear",
      channel: "RGB",
      tiling: false,
      seam: { enabled: false, sensitivity: 32 },
      mipLevel: 0,
    };

    const state = this.coerceInspectorState(
      loadInspectSection<TextureInspectorStateV1>(this.filePathForState, "texture", fallback),
      fallback
    );

    this.preset = state.preset;
    this.sampling = state.sampling;
    this.channel = state.channel;
    this.tiling = state.tiling;
    this.seam = state.seam.enabled;
    this.seamSensitivity = state.seam.sensitivity;
    this.mipLevel = Math.max(0, Math.min(this.maxMipLevel, state.mipLevel));

    this.tileBtn.style.background = this.tiling ? "#007acc" : "#444";
    this.seamBtn.style.background = this.seam ? "#007acc" : "#444";
    this.seamSensitivitySlider.style.display = this.seam ? "block" : "none";
    this.seamSensitivitySlider.value = String(this.seamSensitivity);

    this.presetSelect.value = this.preset === "pixel" ? "Pixel" : "Material";
    this.samplingSelect.value = this.sampling === "linear" ? "Linear" : "Nearest";
    this.channelSelect.value = this.channel;
    this.populateMipSelect(this.maxMipLevel);
    this.mipSelect.value = `Mip ${this.mipLevel}`;

    // Clear hover/pin since image changed.
    this.hoverTexel = null;
    this.pinnedTexel = null;
    this.updatePixelReadout();
  }

  private coerceInspectorState(
    input: TextureInspectorStateV1,
    fallback: TextureInspectorStateV1
  ): TextureInspectorStateV1 {
    if (!input || input.v !== 1) return fallback;

    const preset: TexturePreset = input.preset === "pixel" || input.preset === "material" ? input.preset : fallback.preset;
    const sampling: TextureSampling =
      input.sampling === "nearest" || input.sampling === "linear" ? input.sampling : fallback.sampling;
    const channel: TextureChannel =
      input.channel === "RGB" || input.channel === "R" || input.channel === "G" || input.channel === "B" || input.channel === "A"
        ? input.channel
        : fallback.channel;

    const tiling = typeof input.tiling === "boolean" ? input.tiling : fallback.tiling;
    const seamEnabled = typeof input.seam?.enabled === "boolean" ? input.seam.enabled : fallback.seam.enabled;
    const seamSensitivityRaw = typeof input.seam?.sensitivity === "number" ? input.seam.sensitivity : fallback.seam.sensitivity;
    const seamSensitivity = Math.max(0, Math.min(255, Math.round(seamSensitivityRaw)));

    const mipLevelRaw = typeof input.mipLevel === "number" ? input.mipLevel : fallback.mipLevel;
    const mipLevel = Math.max(0, Math.round(mipLevelRaw));

    return {
      v: 1,
      preset,
      sampling,
      channel,
      tiling,
      seam: { enabled: seamEnabled, sensitivity: seamSensitivity },
      mipLevel,
    };
  }

  private defaultPresetForAssetType(assetType: string | null): TexturePreset {
    const at = (assetType ?? "").toLowerCase();
    if (at === "sprite" || at === "ui" || at === "font" || at === "vfx") return "pixel";
    return "material";
  }

  private persistInspectorState(): void {
    const state: TextureInspectorStateV1 = {
      v: 1,
      preset: this.preset,
      sampling: this.sampling,
      channel: this.channel,
      tiling: this.tiling,
      seam: { enabled: this.seam, sensitivity: this.seamSensitivity },
      mipLevel: this.mipLevel,
    };
    saveInspectSection(this.filePathForState, "texture", state);
    this.updateInfo();
  }

  private getMipCanvas(level: number): HTMLCanvasElement {
    if (!this.image) throw new Error("No image loaded");

    const clamped = Math.max(0, Math.min(this.maxMipLevel, level));
    const cached = this.mipCanvasCache.get(clamped);
    if (cached) return cached;

    if (clamped === 0) {
      const base = document.createElement("canvas");
      base.width = this.image.width;
      base.height = this.image.height;
      const baseCtx = base.getContext("2d", { willReadFrequently: true })!;
      baseCtx.imageSmoothingEnabled = false;
      baseCtx.drawImage(this.image, 0, 0);
      this.mipCanvasCache.set(0, base);
      return base;
    }

    const prev = this.getMipCanvas(clamped - 1);
    const next = document.createElement("canvas");
    next.width = Math.max(1, Math.floor(prev.width / 2));
    next.height = Math.max(1, Math.floor(prev.height / 2));
    const nextCtx = next.getContext("2d", { willReadFrequently: true })!;
    nextCtx.imageSmoothingEnabled = true;
    nextCtx.imageSmoothingQuality = "high";
    nextCtx.drawImage(prev, 0, 0, prev.width, prev.height, 0, 0, next.width, next.height);
    this.mipCanvasCache.set(clamped, next);
    return next;
  }

  private getFilteredCanvas(level: number, channel: "R" | "G" | "B" | "A"): HTMLCanvasElement {
    const key = `${level}:${channel}`;
    const cached = this.filteredCanvasCache.get(key);
    if (cached) return cached;

    const src = this.getMipCanvas(level);
    const out = document.createElement("canvas");
    out.width = src.width;
    out.height = src.height;

    const outCtx = out.getContext("2d", { willReadFrequently: true })!;
    outCtx.imageSmoothingEnabled = false;
    outCtx.drawImage(src, 0, 0);

    const imgData = outCtx.getImageData(0, 0, out.width, out.height);
    const data = imgData.data;
    const offset = channel === "R" ? 0 : channel === "G" ? 1 : channel === "B" ? 2 : 3;
    for (let i = 0; i < data.length; i += 4) {
      const v = data[i + offset];
      data[i + 0] = v;
      data[i + 1] = v;
      data[i + 2] = v;
      if (channel === "A") data[i + 3] = 255;
    }
    outCtx.putImageData(imgData, 0, 0);

    this.filteredCanvasCache.set(key, out);
    return out;
  }

  private handleWheel(e: WheelEvent): void {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    this.setZoom(this.zoom * factor);
  }

  private handleMouseDown(e: MouseEvent): void {
    if (e.button !== 0) return;
    this.isPointerDown = true;
    this.isPanning = false;
    this.pointerDownClientX = e.clientX;
    this.pointerDownClientY = e.clientY;
    this.lastMouseX = e.clientX;
    this.lastMouseY = e.clientY;
  }

  private handleMouseMove(e: MouseEvent): void {
    const pos = this.getMousePos(e);
    this.updateHoverTexel(pos.x, pos.y);
    this.scheduleOverlayDraw();

    if (!this.isPointerDown) return;

    const dx0 = e.clientX - this.pointerDownClientX;
    const dy0 = e.clientY - this.pointerDownClientY;
    if (!this.isPanning) {
      if (Math.abs(dx0) + Math.abs(dy0) >= 3) {
        this.isPanning = true;
        this.canvas.style.cursor = "grabbing";
      } else {
        return;
      }
    }

    const dx = e.clientX - this.lastMouseX;
    const dy = e.clientY - this.lastMouseY;
    this.panX += dx;
    this.panY += dy;
    this.lastMouseX = e.clientX;
    this.lastMouseY = e.clientY;
    this.draw();
  }

  private handleMouseUp(): void {
    if (this.isPointerDown && !this.isPanning) {
      // Click-to-pin.
      if (this.hoverTexel) {
        this.pinnedTexel = { ...this.hoverTexel };
        this.updatePixelReadout();
        this.scheduleOverlayDraw();
      }
    }

    this.isPointerDown = false;
    this.isPanning = false;
    this.canvas.style.cursor = "grab";
  }

  private handleResize(): void {
    this.dpr = Math.max(1, Math.floor(window.devicePixelRatio || 1));
    this.canvasCssWidth = Math.max(1, this.container.clientWidth - 24);
    this.canvasCssHeight = Math.max(100, this.container.clientHeight - 80);

    const pxW = Math.max(1, Math.round(this.canvasCssWidth * this.dpr));
    const pxH = Math.max(1, Math.round(this.canvasCssHeight * this.dpr));
    this.canvas.width = pxW;
    this.canvas.height = pxH;
    this.overlayCanvas.width = pxW;
    this.overlayCanvas.height = pxH;

    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.overlayCtx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.draw();
  }

  private drawEmpty(): void {
    const width = this.canvasCssWidth;
    const height = this.canvasCssHeight;

    this.ctx.imageSmoothingEnabled = false;
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

    this.clearOverlay();
  }

  private draw(): void {
    if (!this.image) {
      this.drawEmpty();
      return;
    }

    const width = this.canvasCssWidth;
    const height = this.canvasCssHeight;

    // Display sampling.
    this.ctx.imageSmoothingEnabled = this.sampling === "linear";
    this.ctx.imageSmoothingQuality = this.sampling === "linear" ? "high" : "low";

    // Draw checkerboard background
    const size = 16;
    for (let y = 0; y < height; y += size) {
      for (let x = 0; x < width; x += size) {
        const isLight = ((x / size) + (y / size)) % 2 === 0;
        this.ctx.fillStyle = isLight ? "#2a2a2a" : "#222";
        this.ctx.fillRect(x, y, size, size);
      }
    }

    const source =
      this.channel === "RGB"
        ? this.getMipCanvas(this.mipLevel)
        : this.getFilteredCanvas(this.mipLevel, this.channel);

    const imgW = source.width * this.zoom;
    const imgH = source.height * this.zoom;

    if (this.compareEnabled && this.compareSnapshot && !this.seam) {
      const drawSource = (src: CanvasImageSource): void => {
        if (this.tiling) {
          const startX = ((this.panX % imgW) + imgW) % imgW - imgW;
          const startY = ((this.panY % imgH) + imgH) % imgH - imgH;

          for (let y = startY; y < height; y += imgH) {
            for (let x = startX; x < width; x += imgW) {
              this.ctx.drawImage(src, x, y, imgW, imgH);
            }
          }
        } else {
          const x = (width - imgW) / 2 + this.panX;
          const y = (height - imgH) / 2 + this.panY;
          this.ctx.drawImage(src, x, y, imgW, imgH);
        }
      };

      // Draw A (snapshot) full, then clip and draw B (live) on top.
      drawSource(this.compareSnapshot.canvas);

      const splitX = wipeToSplitX(this.compareWipe, width);
      this.ctx.save();
      this.ctx.beginPath();
      this.ctx.rect(splitX, 0, width - splitX, height);
      this.ctx.clip();
      drawSource(source);
      this.ctx.restore();

      // Vertical split line (high contrast).
      const lineX = Math.round(splitX) + 0.5;
      this.ctx.save();
      this.ctx.lineWidth = 1;
      this.ctx.strokeStyle = "rgba(0, 0, 0, 0.85)";
      this.ctx.beginPath();
      this.ctx.moveTo(lineX - 1, 0);
      this.ctx.lineTo(lineX - 1, height);
      this.ctx.stroke();
      this.ctx.strokeStyle = "rgba(255, 255, 255, 0.95)";
      this.ctx.beginPath();
      this.ctx.moveTo(lineX, 0);
      this.ctx.lineTo(lineX, height);
      this.ctx.stroke();
      this.ctx.restore();

      this.scheduleOverlayDraw();
      return;
    }

    if (this.seam) {
      const baseX = (width - imgW) / 2 + this.panX;
      const baseY = (height - imgH) / 2 + this.panY;

      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          const x = baseX + dx * imgW;
          const y = baseY + dy * imgH;
          this.ctx.drawImage(source, x, y, imgW, imgH);
        }
      }

      const left = baseX - imgW;
      const top = baseY - imgH;

      this.ctx.save();
      this.ctx.lineWidth = 1;
      this.ctx.strokeStyle = "rgba(0, 0, 0, 0.55)";
      for (let dy = 0; dy < 3; dy++) {
        for (let dx = 0; dx < 3; dx++) {
          const x = Math.round(left + dx * imgW) + 0.5;
          const y = Math.round(top + dy * imgH) + 0.5;
          this.ctx.strokeRect(x, y, Math.round(imgW), Math.round(imgH));
        }
      }
      this.ctx.strokeStyle = "rgba(255, 255, 255, 0.25)";
      this.ctx.strokeRect(Math.round(left) + 0.5, Math.round(top) + 0.5, Math.round(imgW * 3), Math.round(imgH * 3));

      this.drawSeamEdgeDiffOverlay(baseX, baseY, imgW, imgH);
      this.ctx.restore();
    }

    if (this.tiling) {
      // Draw tiled pattern
      const startX = ((this.panX % imgW) + imgW) % imgW - imgW;
      const startY = ((this.panY % imgH) + imgH) % imgH - imgH;

      for (let y = startY; y < height; y += imgH) {
        for (let x = startX; x < width; x += imgW) {
          this.ctx.drawImage(source, x, y, imgW, imgH);
        }
      }
    } else {
      // Draw single image centered with pan offset
      const x = (width - imgW) / 2 + this.panX;
      const y = (height - imgH) / 2 + this.panY;
      this.ctx.drawImage(source, x, y, imgW, imgH);
    }

    this.scheduleOverlayDraw();
  }

  private getMousePos(e: MouseEvent): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  }

  private scheduleOverlayDraw(): void {
    if (this.pendingOverlayRaf !== null) return;
    this.pendingOverlayRaf = window.requestAnimationFrame(() => {
      this.pendingOverlayRaf = null;
      this.drawOverlay();
    });
  }

  private clearOverlay(): void {
    this.overlayCtx.clearRect(0, 0, this.canvasCssWidth, this.canvasCssHeight);
  }

  private drawOverlay(): void {
    this.clearOverlay();
    if (!this.image) return;

    const pos = this.pinnedTexel ?? this.hoverTexel;
    if (!pos) return;

    const hit = this.texelToCanvasPoint(pos);
    if (!hit) return;

    const { x, y } = hit;

    // Crosshair
    this.overlayCtx.save();
    this.overlayCtx.lineWidth = 1;
    this.overlayCtx.strokeStyle = "rgba(255, 255, 255, 0.35)";
    this.overlayCtx.beginPath();
    this.overlayCtx.moveTo(Math.round(x) + 0.5, 0);
    this.overlayCtx.lineTo(Math.round(x) + 0.5, this.canvasCssHeight);
    this.overlayCtx.moveTo(0, Math.round(y) + 0.5);
    this.overlayCtx.lineTo(this.canvasCssWidth, Math.round(y) + 0.5);
    this.overlayCtx.stroke();

    // Pin marker
    if (this.pinnedTexel) {
      this.overlayCtx.fillStyle = "rgba(255, 204, 0, 0.9)";
      this.overlayCtx.beginPath();
      this.overlayCtx.arc(x, y, 3, 0, Math.PI * 2);
      this.overlayCtx.fill();
    }

    this.overlayCtx.restore();
  }

  private updateHoverTexel(canvasX: number, canvasY: number): void {
    const hit = this.getTexelAtCanvasPos(canvasX, canvasY);
    this.hoverTexel = hit;
    this.updatePixelReadout();
  }

  private updatePixelReadout(): void {
    const pos = this.pinnedTexel ?? this.hoverTexel;
    if (!pos || !this.image) {
      this.pixelInfoDiv.textContent = "";
      return;
    }

    const rgba = this.sampleTexelRGBA(pos.mip, pos.x, pos.y);
    if (!rgba) {
      this.pixelInfoDiv.textContent = "";
      return;
    }

    const w = this.getMipCanvas(pos.mip).width;
    const h = this.getMipCanvas(pos.mip).height;
    const uvx = (pos.x + 0.5) / w;
    const uvy = (pos.y + 0.5) / h;

    const hex = `#${rgba.r.toString(16).padStart(2, "0")}${rgba.g
      .toString(16)
      .padStart(2, "0")}${rgba.b.toString(16).padStart(2, "0")}${rgba.a
      .toString(16)
      .padStart(2, "0")}`.toUpperCase();

    const vec4 = `vec4(${(rgba.r / 255).toFixed(3)}, ${(rgba.g / 255).toFixed(3)}, ${(rgba.b / 255).toFixed(
      3
    )}, ${(rgba.a / 255).toFixed(3)})`;

    const label = this.pinnedTexel ? "PIN" : "HOVER";
    this.pixelInfoDiv.textContent = `${label} m${pos.mip} x${pos.x} y${pos.y} uv(${uvx.toFixed(3)},${uvy.toFixed(
      3
    )}) rgba(${rgba.r},${rgba.g},${rgba.b},${rgba.a}) ${hex} ${vec4}`;
  }

  private getTexelAtCanvasPos(canvasX: number, canvasY: number): TexelCoord | null {
    if (!this.image) return null;

    const source =
      this.channel === "RGB"
        ? this.getMipCanvas(this.mipLevel)
        : this.getFilteredCanvas(this.mipLevel, this.channel);

    const width = this.canvasCssWidth;
    const height = this.canvasCssHeight;
    const imgW = source.width * this.zoom;
    const imgH = source.height * this.zoom;

    // Seam view draws a 3x3 grid; treat it as tiling hit test centered on baseX/baseY.
    if (this.seam) {
      const baseX = (width - imgW) / 2 + this.panX;
      const baseY = (height - imgH) / 2 + this.panY;
      const localX = this.posMod(canvasX - baseX, imgW);
      const localY = this.posMod(canvasY - baseY, imgH);
      return {
        x: Math.max(0, Math.min(source.width - 1, Math.floor(localX / this.zoom))),
        y: Math.max(0, Math.min(source.height - 1, Math.floor(localY / this.zoom))),
        mip: this.mipLevel,
      };
    }

    if (this.tiling) {
      const startX = this.posMod(this.panX, imgW) - imgW;
      const startY = this.posMod(this.panY, imgH) - imgH;
      const localX = this.posMod(canvasX - startX, imgW);
      const localY = this.posMod(canvasY - startY, imgH);
      return {
        x: Math.max(0, Math.min(source.width - 1, Math.floor(localX / this.zoom))),
        y: Math.max(0, Math.min(source.height - 1, Math.floor(localY / this.zoom))),
        mip: this.mipLevel,
      };
    }

    const x0 = (width - imgW) / 2 + this.panX;
    const y0 = (height - imgH) / 2 + this.panY;
    if (canvasX < x0 || canvasY < y0 || canvasX >= x0 + imgW || canvasY >= y0 + imgH) {
      return null;
    }

    const localX = canvasX - x0;
    const localY = canvasY - y0;

    return {
      x: Math.max(0, Math.min(source.width - 1, Math.floor(localX / this.zoom))),
      y: Math.max(0, Math.min(source.height - 1, Math.floor(localY / this.zoom))),
      mip: this.mipLevel,
    };
  }

  private texelToCanvasPoint(texel: TexelCoord): { x: number; y: number } | null {
    if (!this.image) return null;

    const source =
      this.channel === "RGB"
        ? this.getMipCanvas(this.mipLevel)
        : this.getFilteredCanvas(this.mipLevel, this.channel);

    if (texel.mip !== this.mipLevel) return null;

    const width = this.canvasCssWidth;
    const height = this.canvasCssHeight;
    const imgW = source.width * this.zoom;
    const imgH = source.height * this.zoom;

    // Map to the central tile origin.
    const x0 = (width - imgW) / 2 + this.panX;
    const y0 = (height - imgH) / 2 + this.panY;

    const x = x0 + (texel.x + 0.5) * this.zoom;
    const y = y0 + (texel.y + 0.5) * this.zoom;

    return { x, y };
  }

  private sampleTexelRGBA(mip: number, x: number, y: number): { r: number; g: number; b: number; a: number } | null {
    if (!this.image) return null;

    const img = this.getMipImageData(mip);
    if (!img) return null;
    const w = img.width;
    const h = img.height;
    const xx = Math.max(0, Math.min(w - 1, Math.floor(x)));
    const yy = Math.max(0, Math.min(h - 1, Math.floor(y)));
    const idx = (yy * w + xx) * 4;
    const d = img.data;
    return { r: d[idx], g: d[idx + 1], b: d[idx + 2], a: d[idx + 3] };
  }

  private getMipImageData(level: number): ImageData | null {
    const clamped = Math.max(0, Math.min(this.maxMipLevel, level));
    const cached = this.mipImageDataCache.get(clamped);
    if (cached) return cached;

    const canvas = this.getMipCanvas(clamped);
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    const img = ctx.getImageData(0, 0, canvas.width, canvas.height);
    this.mipImageDataCache.set(clamped, img);
    return img;
  }

  private posMod(x: number, m: number): number {
    if (m === 0) return 0;
    return ((x % m) + m) % m;
  }

  private getSeamCacheForMip(level: number): SeamCache | null {
    const clamped = Math.max(0, Math.min(this.maxMipLevel, level));
    const cached = this.seamCache.get(clamped);
    if (cached) return cached;

    const img = this.getMipImageData(clamped);
    if (!img) return null;

    const w = img.width;
    const h = img.height;
    const rowMaxDelta = new Uint8Array(h);
    const colMaxDelta = new Uint8Array(w);
    const d = img.data;

    for (let y = 0; y < h; y++) {
      const li = (y * w + 0) * 4;
      const ri = (y * w + (w - 1)) * 4;
      const dr = Math.abs(d[li + 0] - d[ri + 0]);
      const dg = Math.abs(d[li + 1] - d[ri + 1]);
      const db = Math.abs(d[li + 2] - d[ri + 2]);
      const da = Math.abs(d[li + 3] - d[ri + 3]);
      rowMaxDelta[y] = Math.max(dr, dg, db, da);
    }

    for (let x = 0; x < w; x++) {
      const ti = (0 * w + x) * 4;
      const bi = ((h - 1) * w + x) * 4;
      const dr = Math.abs(d[ti + 0] - d[bi + 0]);
      const dg = Math.abs(d[ti + 1] - d[bi + 1]);
      const db = Math.abs(d[ti + 2] - d[bi + 2]);
      const da = Math.abs(d[ti + 3] - d[bi + 3]);
      colMaxDelta[x] = Math.max(dr, dg, db, da);
    }

    const out: SeamCache = { width: w, height: h, rowMaxDelta, colMaxDelta };
    this.seamCache.set(clamped, out);
    return out;
  }

  private drawSeamEdgeDiffOverlay(baseX: number, baseY: number, imgW: number, imgH: number): void {
    const cache = this.getSeamCacheForMip(this.mipLevel);
    if (!cache) return;
    if (cache.width <= 1 || cache.height <= 1) return;

    const threshold = this.seamSensitivity;
    const rightX = baseX + imgW;
    const bottomY = baseY + imgH;

    this.ctx.save();
    this.ctx.strokeStyle = "rgba(255, 80, 80, 0.85)";
    this.ctx.lineWidth = 2;

    // Right edge highlights (left vs right columns)
    for (let y = 0; y < cache.height; y++) {
      if (cache.rowMaxDelta[y] < threshold) continue;
      const yy = baseY + (y + 0.5) * this.zoom;
      this.ctx.beginPath();
      this.ctx.moveTo(Math.round(rightX) + 0.5, Math.round(yy - this.zoom * 0.4) + 0.5);
      this.ctx.lineTo(Math.round(rightX) + 0.5, Math.round(yy + this.zoom * 0.4) + 0.5);
      this.ctx.stroke();
    }

    // Bottom edge highlights (top vs bottom rows)
    for (let x = 0; x < cache.width; x++) {
      if (cache.colMaxDelta[x] < threshold) continue;
      const xx = baseX + (x + 0.5) * this.zoom;
      this.ctx.beginPath();
      this.ctx.moveTo(Math.round(xx - this.zoom * 0.4) + 0.5, Math.round(bottomY) + 0.5);
      this.ctx.lineTo(Math.round(xx + this.zoom * 0.4) + 0.5, Math.round(bottomY) + 0.5);
      this.ctx.stroke();
    }

    this.ctx.restore();
  }

  /**
   * Clear the preview.
   */
  clear(): void {
    this.image = null;
    this.clearDerivedCaches();
    this.seam = false;
    this.seamBtn.style.background = "#444";
    this.seamSensitivitySlider.style.display = "none";

    this.compareEnabled = false;
    this.compareSnapshot = null;
    this.updateCompareControls();

    this.channel = "RGB";
    this.channelSelect.value = this.channel;
    this.mipLevel = 0;
    this.maxMipLevel = 0;

    this.hoverTexel = null;
    this.pinnedTexel = null;
    this.pixelInfoDiv.textContent = "";

    this.populateMipSelect(0);
    this.mipSelect.value = "Mip 0";

    this.zoom = 1;
    this.panX = 0;
    this.panY = 0;
    this.isPanning = false;
    this.isPointerDown = false;
    this.canvas.style.cursor = "grab";
    this.updateInfo();
    this.drawEmpty();
  }

  /**
   * Dispose of the preview and release resources.
   */
  dispose(): void {
    this.clear();

    // Remove event listeners
    this.canvas.removeEventListener("wheel", this.boundHandleWheel, this.wheelListenerOptions);
    this.canvas.removeEventListener("mousedown", this.boundHandleMouseDown);
    this.canvas.removeEventListener("mousemove", this.boundHandleMouseMove);
    this.canvas.removeEventListener("mouseup", this.boundHandleMouseUp);
    this.canvas.removeEventListener("mouseleave", this.boundHandleMouseUp);

    // Disconnect resize observer
    this.resizeObserver.disconnect();

    // Clear DOM
    this.container.innerHTML = "";
  }
}
