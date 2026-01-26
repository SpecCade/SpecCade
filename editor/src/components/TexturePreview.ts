/**
 * Texture preview component with zoom, pan, and tiling support.
 */
export class TexturePreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private controlsDiv: HTMLDivElement;
  private infoDiv: HTMLDivElement;
  private tileBtn: HTMLButtonElement;
  private seamBtn: HTMLButtonElement;
  private channelSelect: HTMLSelectElement;
  private mipSelect: HTMLSelectElement;
  private resizeObserver: ResizeObserver;

  private wheelListenerOptions: AddEventListenerOptions = { passive: false };

  // Bound event handlers for cleanup
  private boundHandleWheel: (e: WheelEvent) => void;
  private boundHandleMouseDown: (e: MouseEvent) => void;
  private boundHandleMouseMove: (e: MouseEvent) => void;
  private boundHandleMouseUp: () => void;

  private image: HTMLImageElement | null = null;
  private zoom = 1;
  private panX = 0;
  private panY = 0;
  private tiling = false;
  private seam = false;
  private channel: "RGB" | "R" | "G" | "B" | "A" = "RGB";
  private mipLevel = 0;
  private maxMipLevel = 0;
  private isPanning = false;
  private lastMouseX = 0;
  private lastMouseY = 0;

  private mipCanvasCache = new Map<number, HTMLCanvasElement>();
  private filteredCanvasCache = new Map<string, HTMLCanvasElement>();

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
    this.tileBtn = this.createButton("Tile", () => this.toggleTiling());
    this.seamBtn = this.createButton("Seam", () => this.toggleSeam());

    this.channelSelect = this.createSelect(["RGB", "R", "G", "B", "A"], (value) => {
      this.channel = value as typeof this.channel;
      this.draw();
    });

    this.mipSelect = this.createSelect(["Mip 0"], (value) => {
      const m = Number.parseInt(value.replace(/^Mip\s+/, ""), 10);
      this.mipLevel = Number.isFinite(m) ? Math.max(0, Math.min(this.maxMipLevel, m)) : 0;
      this.draw();
    });

    this.controlsDiv.appendChild(zoomOutBtn);
    this.controlsDiv.appendChild(zoomInBtn);
    this.controlsDiv.appendChild(resetBtn);
    this.controlsDiv.appendChild(this.tileBtn);
    this.controlsDiv.appendChild(this.seamBtn);
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
  loadTexture(base64Data: string, mimeType: string, metadata?: Record<string, unknown>): Promise<void> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        this.image = img;
        this.clearDerivedCaches();
        this.maxMipLevel = this.computeMaxMipLevel(img.width, img.height);
        this.mipLevel = 0;
        this.channel = "RGB";
        this.seam = false;
        this.seamBtn.style.background = "#444";
        this.channelSelect.value = this.channel;
        this.populateMipSelect(this.maxMipLevel);
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
    this.tileBtn.style.background = this.tiling ? "#007acc" : "#444";
    this.draw();
  }

  private toggleSeam(): void {
    this.seam = !this.seam;
    this.seamBtn.style.background = this.seam ? "#007acc" : "#444";
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
  }

  private draw(): void {
    if (!this.image) {
      this.drawEmpty();
      return;
    }

    const { width, height } = this.canvas;

    // Keep preview crisp; mip generation handles its own filtering.
    this.ctx.imageSmoothingEnabled = false;

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
      this.ctx.restore();
      return;
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
  }

  /**
   * Clear the preview.
   */
  clear(): void {
    this.image = null;
    this.clearDerivedCaches();
    this.seam = false;
    this.seamBtn.style.background = "#444";
    this.channel = "RGB";
    this.channelSelect.value = this.channel;
    this.mipLevel = 0;
    this.maxMipLevel = 0;

    this.populateMipSelect(0);
    this.mipSelect.value = "Mip 0";

    this.zoom = 1;
    this.panX = 0;
    this.panY = 0;
    this.isPanning = false;
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
