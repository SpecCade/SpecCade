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
