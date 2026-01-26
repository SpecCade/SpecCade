import { ChiptuneJsPlayer } from "chiptune3";
import { loadInspectSection, saveInspectSection } from "../lib/storage";
import { clampOrderRange } from "../lib/music-loop";

type LoopMode = "song" | "order" | "order_range" | "off";

type MusicInspectorStateV1 = {
  v: 1;
  loopMode: LoopMode;
  loopStartOrder: number;
  loopEndOrder: number;
};

export type MusicPreviewResult = {
  dataBase64: string;
  mimeType?: string;
  metadata?: Record<string, unknown>;
};

export type MusicPreviewRequest = (source: string, filename: string) => Promise<MusicPreviewResult>;

type RefreshMode = {
  autoplay: boolean;
  keepPaused?: boolean;
};

const LIVE_PREVIEW_STORAGE_KEY = "speccade-music-live-preview";

/**
 * Music preview component for tracker module formats (XM/IT/etc).
 *
 * Fetching/regeneration is provided via `requestPreview` to keep backend concerns out.
 */
export class MusicPreview {
  private container: HTMLElement;
  private wrapper: HTMLDivElement;
  private player: ChiptuneJsPlayer;
  private playerReady: Promise<void>;
  private requestPreview: MusicPreviewRequest;

  private playButton: HTMLButtonElement;
  private stopButton: HTMLButtonElement;
  private refreshButton: HTMLButtonElement;
  private seekSlider: HTMLInputElement;
  private volumeSlider: HTMLInputElement;
  private livePreviewCheckbox: HTMLInputElement;
  private infoText: HTMLDivElement;

  private currentTimeText: HTMLDivElement;
  private totalTimeText: HTMLDivElement;

  private nowPlayingText: HTMLDivElement;
  private orderStrip: HTMLDivElement;

  private loopModeSelect: HTMLSelectElement;
  private loopStartSelect: HTMLSelectElement;
  private loopEndSelect: HTMLSelectElement;
  private totalOrders: number | null = null;

  private loopMode: LoopMode = "song";
  private loopStartOrder = 0;
  private loopEndOrder = 0;
  private lastLoopSeekAtMs = 0;
  private pendingSeekOrder: number | null = null;

  private orderStartSecByOrder = new Map<number, number>();
  private orderPatternByOrder = new Map<number, number>();
  private orderList: number[] = [];
  private orderChipByOrder = new Map<number, HTMLButtonElement>();
  private nowPlaying = { order: null as number | null, pattern: null as number | null, row: null as number | null };

  private lastHighlightKey: string | null = null;

  private refreshSeq = 0;
  private isDisposed = false;
  private isPlaying = false;
  private isPaused = false;
  private isSeeking = false;
  private durationSec: number | null = null;
  private currentBuffer: ArrayBuffer | null = null;
  private currentBufferKey: string | null = null;
  private currentMimeType: string | undefined;
  private source: string | null = null;
  private filename: string | null = null;
  private isDirty = false;

  private onWindowPointerUp = () => {
    this.isSeeking = false;
  };

  constructor(container: HTMLElement, requestPreview: MusicPreviewRequest) {
    this.container = container;
    this.requestPreview = requestPreview;

    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      gap: 10px;
      padding: 12px;
      box-sizing: border-box;
    `;

    this.wrapper.tabIndex = 0;
    this.wrapper.addEventListener("keydown", (e) => this.onKeyDown(e));

    const controlsRow = document.createElement("div");
    controlsRow.style.cssText = `
      display: flex;
      gap: 8px;
      align-items: center;
      flex-wrap: wrap;
    `;

    this.playButton = this.makeButton("Play", "#007acc");
    this.playButton.onclick = () => this.onPlayClicked();
    controlsRow.appendChild(this.playButton);

    this.stopButton = this.makeButton("Stop", "#444");
    this.stopButton.onclick = () => this.stop();
    controlsRow.appendChild(this.stopButton);

    this.refreshButton = this.makeButton("Refresh", "#2a6f4a");
    this.refreshButton.onclick = () => {
      void this.refresh({ autoplay: this.isPlaying, keepPaused: this.isPaused });
    };
    controlsRow.appendChild(this.refreshButton);

    const spacer = document.createElement("div");
    spacer.style.flex = "1";
    controlsRow.appendChild(spacer);

    const liveLabel = document.createElement("label");
    liveLabel.style.cssText = `
      display: flex;
      gap: 6px;
      align-items: center;
      font-size: 12px;
      color: #c7c7c7;
      user-select: none;
    `;

    this.livePreviewCheckbox = document.createElement("input");
    this.livePreviewCheckbox.type = "checkbox";
    this.livePreviewCheckbox.checked = this.loadLivePreviewEnabled();
    this.livePreviewCheckbox.onchange = () => {
      this.storeLivePreviewEnabled(this.livePreviewCheckbox.checked);
    };
    liveLabel.appendChild(this.livePreviewCheckbox);
    liveLabel.appendChild(document.createTextNode("Live preview"));
    controlsRow.appendChild(liveLabel);

    this.wrapper.appendChild(controlsRow);

    const sliders = document.createElement("div");
    sliders.style.cssText = `
      display: flex;
      flex-direction: column;
      gap: 8px;
      padding: 8px 10px;
      background: #1e1e1e;
      border-radius: 6px;
    `;

    const seekRow = document.createElement("div");
    seekRow.style.cssText = `
      display: grid;
      grid-template-columns: 52px 1fr 52px;
      gap: 8px;
      align-items: center;
      color: #bdbdbd;
      font-size: 11px;
    `;

    const seekLeft = document.createElement("div");
    seekLeft.textContent = "0:00";
    seekLeft.id = "music-current";
    seekLeft.style.textAlign = "right";
    this.currentTimeText = seekLeft;
    seekRow.appendChild(seekLeft);

    this.seekSlider = document.createElement("input");
    this.seekSlider.type = "range";
    this.seekSlider.min = "0";
    this.seekSlider.max = "1";
    this.seekSlider.step = "0.01";
    this.seekSlider.value = "0";
    this.seekSlider.style.width = "100%";
    this.seekSlider.addEventListener("pointerdown", () => {
      this.isSeeking = true;
    });
    this.seekSlider.addEventListener("input", () => {
      if (!this.isPlaying) return;
      let sec = Number(this.seekSlider.value);
      if (!Number.isFinite(sec)) return;
      sec = Math.max(0, sec);
      if (typeof this.durationSec === "number" && Number.isFinite(this.durationSec) && this.durationSec > 0) {
        sec = Math.min(sec, this.durationSec);
      }

      this.seekSlider.value = String(sec);
      this.updateTimeText(sec, this.durationSec ?? 0);
      try {
        this.player.setPos(sec);
      } catch {
        // ignore
      }
    });
    seekRow.appendChild(this.seekSlider);

    const seekRight = document.createElement("div");
    seekRight.textContent = "0:00";
    seekRight.id = "music-total";
    seekRight.style.textAlign = "left";
    this.totalTimeText = seekRight;
    seekRow.appendChild(seekRight);

    sliders.appendChild(seekRow);

    const volRow = document.createElement("div");
    volRow.style.cssText = `
      display: grid;
      grid-template-columns: 52px 1fr 52px;
      gap: 8px;
      align-items: center;
      color: #bdbdbd;
      font-size: 11px;
    `;

    const volLabel = document.createElement("div");
    volLabel.textContent = "Vol";
    volLabel.style.textAlign = "right";
    volRow.appendChild(volLabel);

    this.volumeSlider = document.createElement("input");
    this.volumeSlider.type = "range";
    this.volumeSlider.min = "0";
    this.volumeSlider.max = "1";
    this.volumeSlider.step = "0.01";
    this.volumeSlider.value = "0.8";
    this.volumeSlider.style.width = "100%";
    this.volumeSlider.addEventListener("input", () => {
      this.player.setVol(Number(this.volumeSlider.value));
    });
    volRow.appendChild(this.volumeSlider);

    const volValue = document.createElement("div");
    volValue.textContent = "80%";
    volValue.style.textAlign = "left";
    volRow.appendChild(volValue);

    this.volumeSlider.addEventListener("input", () => {
      volValue.textContent = `${Math.round(Number(this.volumeSlider.value) * 100)}%`;
    });

    sliders.appendChild(volRow);

    this.wrapper.appendChild(sliders);

    const inspector = document.createElement("div");
    inspector.style.cssText = `
      display: flex;
      flex-direction: column;
      gap: 8px;
      padding: 8px 10px;
      background: #1e1e1e;
      border-radius: 6px;
    `;

    this.nowPlayingText = document.createElement("div");
    this.nowPlayingText.style.cssText = `
      font-size: 11px;
      color: #bdbdbd;
      line-height: 1.3;
      user-select: text;
    `;
    this.nowPlayingText.textContent = this.formatNowPlaying();
    inspector.appendChild(this.nowPlayingText);

    const loopRow = document.createElement("div");
    loopRow.style.cssText = `
      display: flex;
      gap: 8px;
      align-items: center;
      flex-wrap: wrap;
      color: #bdbdbd;
      font-size: 11px;
      user-select: none;
    `;

    const loopLabel = document.createElement("div");
    loopLabel.textContent = "Loop";
    loopLabel.style.minWidth = "34px";
    loopRow.appendChild(loopLabel);

    this.loopModeSelect = this.makeSelect([
      { label: "Song", value: "song" },
      { label: "Order", value: "order" },
      { label: "Range", value: "order_range" },
      { label: "Off", value: "off" },
    ]);
    loopRow.appendChild(this.loopModeSelect);

    this.loopStartSelect = this.makeSelect([{ label: "Start 00", value: "0" }]);
    loopRow.appendChild(this.loopStartSelect);

    this.loopEndSelect = this.makeSelect([{ label: "End 00", value: "0" }]);
    loopRow.appendChild(this.loopEndSelect);

    const setStartCurrentBtn = this.makeMiniButton("Set Start = Current");
    setStartCurrentBtn.onclick = () => this.setLoopStartToCurrent();
    loopRow.appendChild(setStartCurrentBtn);

    const setEndCurrentBtn = this.makeMiniButton("Set End = Current");
    setEndCurrentBtn.onclick = () => this.setLoopEndToCurrent();
    loopRow.appendChild(setEndCurrentBtn);

    const rangeCurrentBtn = this.makeMiniButton("Range = Current");
    rangeCurrentBtn.onclick = () => this.setLoopRangeToCurrent();
    loopRow.appendChild(rangeCurrentBtn);

    const rangeFullBtn = this.makeMiniButton("Range = Full");
    rangeFullBtn.onclick = () => this.setLoopRangeToFull();
    loopRow.appendChild(rangeFullBtn);

    inspector.appendChild(loopRow);

    this.orderStrip = document.createElement("div");
    this.orderStrip.style.cssText = `
      display: flex;
      gap: 6px;
      overflow-x: auto;
      padding: 2px 0;
      -webkit-overflow-scrolling: touch;
    `;
    inspector.appendChild(this.orderStrip);

    this.wrapper.appendChild(inspector);

    this.infoText = document.createElement("div");
    this.infoText.style.cssText = `
      font-size: 11px;
      color: #999;
      line-height: 1.3;
      user-select: text;
    `;
    this.infoText.textContent = "Music preview";
    this.wrapper.appendChild(this.infoText);

    container.appendChild(this.wrapper);

    // Player setup
    this.player = new ChiptuneJsPlayer({ repeatCount: -1 });
    this.player.setVol(Number(this.volumeSlider.value));

    this.playerReady = new Promise((resolve) => {
      this.player.onInitialized(() => resolve());
    });

    this.player.onMetadata((meta) => {
      if (this.isDisposed) return;
      const dur = typeof meta.dur === "number" ? meta.dur : this.player.duration;
      if (typeof dur === "number" && Number.isFinite(dur) && dur > 0) {
        this.durationSec = dur;
        this.seekSlider.max = String(dur);
        this.totalTimeText.textContent = this.formatTime(dur);
      }
      this.updateInfoText(meta);

      this.populateOrdersFromMetadata(meta);
    });

    this.player.onProgress((e) => {
      if (this.isDisposed) return;
      if (!this.isPlaying) return;
      const pos = typeof e.pos === "number" ? e.pos : (this.player.getCurrentTime() ?? 0);

      const order = typeof e.order === "number" && Number.isFinite(e.order) ? e.order : null;
      const pattern = typeof e.pattern === "number" && Number.isFinite(e.pattern) ? e.pattern : null;
      const row = typeof e.row === "number" && Number.isFinite(e.row) ? e.row : null;

      if (order !== null) {
        const isNewOrder = !this.orderChipByOrder.has(order);

        if (isNewOrder) {
          this.orderList.push(order);
          if (Number.isFinite(pos)) this.orderStartSecByOrder.set(order, pos);
          if (pattern !== null) this.orderPatternByOrder.set(order, pattern);
          this.addOrderChip(order);
        } else {
          if (!this.orderStartSecByOrder.has(order) && Number.isFinite(pos)) {
            this.orderStartSecByOrder.set(order, pos);
          }
          if (!this.orderPatternByOrder.has(order) && pattern !== null) {
            this.orderPatternByOrder.set(order, pattern);
            this.updateOrderChipLabel(order);
          }
        }

        this.setNowPlaying(order, pattern, row);
      } else {
        this.setNowPlaying(null, pattern, row);
      }

      if (order !== null) {
        this.enforceLoop(order);
      }

      if (!this.isSeeking) {
        this.seekSlider.value = String(pos);
        this.updateTimeText(pos, this.durationSec ?? 0);
      }
    });

    this.player.onEnded(() => {
      if (this.isDisposed) return;
      this.isPlaying = false;
      this.isPaused = false;
      this.seekSlider.value = "0";
      this.updateTimeText(0, this.durationSec ?? 0);
      this.playButton.textContent = "Play";
      this.resetNowPlaying();
    });

    this.player.onError((err) => {
      if (this.isDisposed) return;
      this.setInfoText(`Music preview error: ${this.describeError(err)}`);
    });

    this.loopModeSelect.addEventListener("change", () => {
      const next = this.loopModeSelect.value as LoopMode;
      if (next === "order") {
        const ord = this.getCurrentOrderForLoop();
        this.loopStartOrder = ord;
        this.loopEndOrder = this.loopStartOrder;
      }
      this.loopMode = next;

      if (
        this.loopMode === "order_range" &&
        this.loopStartOrder === 0 &&
        this.loopEndOrder === 0 &&
        typeof this.totalOrders === "number" &&
        Number.isFinite(this.totalOrders) &&
        this.totalOrders > 0
      ) {
        this.loopEndOrder = Math.max(0, this.totalOrders - 1);
      }

      this.clampLoopOrders();
      this.updateLoopUI();
      this.persistMusicInspectorState();
      this.applyLoopModeToPlayer();
      this.updateOrderChipHighlights();
    });

    this.loopStartSelect.addEventListener("change", () => {
      const n = Number(this.loopStartSelect.value);
      if (Number.isFinite(n)) this.loopStartOrder = Math.max(0, Math.trunc(n));
      if (this.loopMode === "order") this.loopEndOrder = this.loopStartOrder;
      if (this.loopEndOrder < this.loopStartOrder) this.loopEndOrder = this.loopStartOrder;
      this.clampLoopOrders();
      this.updateLoopUI();
      this.persistMusicInspectorState();
      this.updateOrderChipHighlights();
    });

    this.loopEndSelect.addEventListener("change", () => {
      const n = Number(this.loopEndSelect.value);
      if (Number.isFinite(n)) this.loopEndOrder = Math.max(0, Math.trunc(n));
      if (this.loopEndOrder < this.loopStartOrder) this.loopEndOrder = this.loopStartOrder;
      this.clampLoopOrders();
      this.updateLoopUI();
      this.persistMusicInspectorState();
      this.updateOrderChipHighlights();
    });

    window.addEventListener("pointerup", this.onWindowPointerUp);
    window.addEventListener("pointercancel", this.onWindowPointerUp);
  }

  onSourceUpdated(): void {
    if (!this.source || !this.filename) return;
    this.markDirty();
    if (!this.loadLivePreviewEnabled()) {
      this.setInfoText(this.describeReadyText());
      return;
    }
    void this.refresh({ autoplay: this.isPlaying, keepPaused: this.isPaused });
  }

  setSource(source: string, filename: string): void {
    this.source = source;
    this.filename = filename;

    this.applyMusicInspectorStateFromStorage();
    this.isDirty = this.currentBuffer !== null && this.currentBufferKey !== this.makeSourceKey(source, filename);
    this.setInfoText(this.describeReadyText());
  }

  async setPreviewBytes(base64: string, mimeType?: string, loadedKey?: string | null): Promise<void> {
    const buf = this.decodeBase64(base64);
    this.currentBuffer = buf;
    this.currentMimeType = mimeType;
    this.currentBufferKey = loadedKey ?? this.getDesiredSourceKey();
    this.isDirty = false;
    this.durationSec = null;
    this.seekSlider.max = "1";
    this.seekSlider.value = "0";
    this.updateTimeText(0, 0);
    this.setInfoText(this.describeReadyText());

    this.clearOrderMaps();
    this.resetNowPlaying();
  }

  dispose(): void {
    if (this.isDisposed) return;
    this.isDisposed = true;

    window.removeEventListener("pointerup", this.onWindowPointerUp);
    window.removeEventListener("pointercancel", this.onWindowPointerUp);

    this.stop();
    try {
      this.player.context.close();
    } catch {
      // ignore
    }

    if (this.wrapper.parentElement === this.container) {
      this.container.removeChild(this.wrapper);
    }
  }

  private makeButton(label: string, bg: string): HTMLButtonElement {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = label;
    b.style.cssText = `
      padding: 6px 14px;
      background: ${bg};
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
      font-weight: 500;
    `;
    return b;
  }

  private makeSelect(options: Array<{ label: string; value: string }>): HTMLSelectElement {
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

    for (const optInfo of options) {
      const opt = document.createElement("option");
      opt.value = optInfo.value;
      opt.textContent = optInfo.label;
      select.appendChild(opt);
    }

    return select;
  }

  private makeMiniButton(label: string): HTMLButtonElement {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = label;
    b.style.cssText = `
      padding: 4px 10px;
      background: #2a2a2a;
      color: #cfcfcf;
      border: 1px solid #3a3a3a;
      border-radius: 4px;
      cursor: pointer;
      font-size: 11px;
      line-height: 1;
      height: 26px;
    `;
    return b;
  }

  private onPlayClicked(): void {
    if (this.isPlaying) {
      if (this.isPaused) {
        if (this.isDirty) {
          // Regenerate instead of resuming stale audio.
          void this.refresh({ autoplay: true, keepPaused: false });
          return;
        }

        this.player.togglePause();
        this.isPaused = false;
        this.playButton.textContent = "Pause";
        return;
      }

      this.player.togglePause();
      this.isPaused = true;
      this.playButton.textContent = "Play";
      return;
    }

    if (this.currentBuffer && !this.isDirty) {
      void this.startPlaybackFromCurrentBuffer();
      return;
    }

    void this.refresh({ autoplay: true });
  }

  private stop(): void {
    if (!this.isPlaying && this.currentBuffer === null) return;
    this.player.stop();
    this.isPlaying = false;
    this.isPaused = false;
    this.seekSlider.value = "0";
    this.updateTimeText(0, this.durationSec ?? 0);
    this.playButton.textContent = "Play";
    this.resetNowPlaying();
  }

  private clearOrderMaps(): void {
    this.orderStartSecByOrder.clear();
    this.orderPatternByOrder.clear();
    this.orderList = [];
    this.orderChipByOrder.clear();
    while (this.orderStrip.firstChild) this.orderStrip.removeChild(this.orderStrip.firstChild);
    this.totalOrders = null;
    this.lastHighlightKey = null;
  }

  private populateOrdersFromMetadata(meta: Record<string, unknown>): void {
    const song = (meta as any).song;
    const orders = song?.orders;
    if (!Array.isArray(orders) || orders.length === 0) return;

    this.clearOrderMaps();

    for (let i = 0; i < orders.length; i++) {
      this.orderList.push(i);
      const pat = orders[i]?.pat;
      if (typeof pat === "number" && Number.isFinite(pat)) {
        this.orderPatternByOrder.set(i, pat);
      }
      this.addOrderChip(i, { scroll: false });
    }

    this.totalOrders = orders.length;

    // If no explicit range yet and we're in range mode, default to full span.
    if (this.loopMode === "order_range" && this.loopStartOrder === 0 && this.loopEndOrder === 0) {
      this.loopEndOrder = Math.max(0, orders.length - 1);
    }

    this.clampLoopOrders();
    this.updateLoopUI();
    this.persistMusicInspectorState();
    this.applyLoopModeToPlayer();
    this.updateOrderChipHighlights();
  }

  private resetNowPlaying(): void {
    this.nowPlaying = { order: null, pattern: null, row: null };
    this.nowPlayingText.textContent = this.formatNowPlaying();
    this.updateOrderChipHighlights();
  }

  private setNowPlaying(order: number | null, pattern: number | null, row: number | null): void {
    const next = { order, pattern, row };
    const orderChanged = next.order !== this.nowPlaying.order;
    const changed =
      next.order !== this.nowPlaying.order ||
      next.pattern !== this.nowPlaying.pattern ||
      next.row !== this.nowPlaying.row;
    if (!changed) return;
    this.nowPlaying = next;
    this.nowPlayingText.textContent = this.formatNowPlaying();
    if (orderChanged) this.updateOrderChipHighlights();
  }

  private formatNowPlaying(): string {
    const ord = this.nowPlaying.order === null ? "--" : this.format2(this.nowPlaying.order);
    const pat = this.nowPlaying.pattern === null ? "--" : this.format2(this.nowPlaying.pattern);
    const row = this.nowPlaying.row === null ? "--" : this.format2(this.nowPlaying.row);
    return `Ord ${ord} | Pat ${pat} | Row ${row}`;
  }

  private format2(n: number): string {
    return String(Math.max(0, Math.trunc(n))).padStart(2, "0");
  }

  private addOrderChip(order: number, opts?: { scroll?: boolean }): void {
    const chip = document.createElement("button");
    chip.type = "button";
    chip.textContent = this.getOrderChipLabel(order);
    chip.style.cssText = `
      padding: 4px 8px;
      background: #2a2a2a;
      color: #cfcfcf;
      border: 1px solid #3a3a3a;
      border-radius: 999px;
      cursor: pointer;
      font-size: 11px;
      line-height: 1;
      white-space: nowrap;
    `;
    chip.onclick = () => this.seekToOrder(order);

    this.orderChipByOrder.set(order, chip);
    this.orderStrip.appendChild(chip);
    if (opts?.scroll !== false) {
      chip.scrollIntoView({ inline: "nearest", block: "nearest" });
    }
    this.updateOrderChipHighlights();
  }

  private updateOrderChipLabel(order: number): void {
    const chip = this.orderChipByOrder.get(order);
    if (!chip) return;
    chip.textContent = this.getOrderChipLabel(order);
  }

  private getOrderChipLabel(order: number): string {
    const pat = this.orderPatternByOrder.get(order);
    const patLabel = typeof pat === "number" ? this.format2(pat) : "?";
    return `O${this.format2(order)} P${patLabel}`;
  }

  private updateOrderChipHighlights(): void {
    const current = this.nowPlaying.order;

    const showRange = this.loopMode === "order" || this.loopMode === "order_range";
    let rangeStart = Math.max(0, Math.trunc(this.loopStartOrder));
    let rangeEnd = Math.max(0, Math.trunc(this.loopEndOrder));
    if (this.loopMode === "order") rangeEnd = rangeStart;

    const total = this.totalOrders;
    if (showRange && typeof total === "number" && Number.isFinite(total) && total > 0) {
      const clamped = clampOrderRange({ start: rangeStart, end: rangeEnd }, total);
      rangeStart = clamped.start;
      rangeEnd = clamped.end;
    } else if (showRange && rangeEnd < rangeStart) {
      rangeEnd = rangeStart;
    }

    const key = `${current ?? "null"}|${this.loopMode}|${rangeStart}|${rangeEnd}|${this.orderChipByOrder.size}`;
    if (key === this.lastHighlightKey) return;
    this.lastHighlightKey = key;

    for (const [order, chip] of this.orderChipByOrder.entries()) {
      const isActive = current !== null && order === current;
      if (isActive) {
        this.applyOrderChipActiveStyle(chip, true);
        continue;
      }

      const inRange = showRange && order >= rangeStart && order <= rangeEnd;
      if (inRange) {
        this.applyOrderChipRangeStyle(chip);
      } else {
        this.applyOrderChipActiveStyle(chip, false);
      }
    }
  }

  private applyOrderChipActiveStyle(chip: HTMLButtonElement, active: boolean): void {
    chip.style.background = active ? "#007acc" : "#2a2a2a";
    chip.style.borderColor = active ? "#007acc" : "#3a3a3a";
    chip.style.color = active ? "#fff" : "#cfcfcf";
  }

  private applyOrderChipRangeStyle(chip: HTMLButtonElement): void {
    chip.style.background = "#203747";
    chip.style.borderColor = "#2e5c7a";
    chip.style.color = "#d7e9f6";
  }

  private seekToOrder(order: number): void {
    // Highlight immediately even if seek fails.
    const pat = this.orderPatternByOrder.get(order);
    this.setNowPlaying(order, typeof pat === "number" ? pat : null, null);

    // If we're not playing yet, remember intent and apply on play.
    if (!this.isPlaying) {
      this.pendingSeekOrder = order;
      return;
    }

    try {
      this.player.setOrderRow(order, 0);
      this.pendingSeekOrder = null;
      return;
    } catch {
      // fallback to time seeking
    }

    const knownStart = this.orderStartSecByOrder.get(order);
    let target: number | null = null;
    if (typeof knownStart === "number" && Number.isFinite(knownStart)) {
      target = knownStart;
    } else if (typeof this.durationSec === "number" && Number.isFinite(this.durationSec) && this.durationSec > 0) {
      const idx = this.orderList.indexOf(order);
      const denom = Math.max(1, this.orderList.length - 1);
      target = this.durationSec * (Math.max(0, idx) / denom);
    }

    if (target === null) return;

    try {
      this.player.setPos(target);
    } catch {
      // ignore
    }

    if (!this.isSeeking) {
      this.seekSlider.value = String(target);
      this.updateTimeText(target, this.durationSec ?? 0);
    }
  }

  private async refresh(mode: RefreshMode): Promise<void> {
    const seq = ++this.refreshSeq;
    this.setControlsEnabled(false);
    this.setInfoText("Generating music preview...");

    try {
      if (!this.source || !this.filename) {
        this.setInfoText("Music preview | no source");
        this.currentBuffer = null;
        this.currentMimeType = undefined;
        this.currentBufferKey = null;
        this.isDirty = false;
        this.stop();
        return;
      }

      const source = this.source;
      const filename = this.filename;
      const sourceKey = this.makeSourceKey(source, filename);

      const result = await this.requestPreview(source, filename);
      if (this.isDisposed || seq !== this.refreshSeq) return;

      // Source changed mid-refresh; do not replace the buffer with stale audio.
      if (this.getDesiredSourceKey() !== sourceKey) {
        this.isDirty = this.currentBuffer !== null;
        this.setInfoText(this.describeReadyText());
        return;
      }

      await this.setPreviewBytes(result.dataBase64, result.mimeType, sourceKey);
      if (this.isDisposed || seq !== this.refreshSeq) return;

      if (mode.autoplay) {
        const keepPaused = mode.keepPaused === true;
        await this.startPlaybackFromCurrentBuffer({ keepPaused, restart: true });
      } else {
        this.isPlaying = false;
        this.isPaused = false;
        this.playButton.textContent = "Play";
        this.setInfoText(this.describeReadyText());
      }
    } catch (e) {
      if (this.isDisposed || seq !== this.refreshSeq) return;
      this.setInfoText(`Failed to generate music preview: ${this.describeError(e)}`);
      this.stop();
    } finally {
      if (!this.isDisposed && seq === this.refreshSeq) {
        this.setControlsEnabled(true);
      }
    }
  }

  private setControlsEnabled(enabled: boolean): void {
    this.playButton.disabled = !enabled;
    this.stopButton.disabled = !enabled;
    this.refreshButton.disabled = !enabled;
    this.seekSlider.disabled = !enabled;
    this.volumeSlider.disabled = !enabled;
    this.livePreviewCheckbox.disabled = !enabled;

    const opacity = enabled ? "1" : "0.6";
    this.playButton.style.opacity = opacity;
    this.stopButton.style.opacity = opacity;
    this.refreshButton.style.opacity = opacity;
  }

  private setInfoText(text: string): void {
    this.infoText.textContent = text;
  }

  private updateInfoText(meta: Record<string, unknown>): void {
    const bits: string[] = [];
    bits.push(this.describeReadyText());

    const title = typeof meta.title === "string" ? meta.title : undefined;
    const artist = typeof meta.artist === "string" ? meta.artist : undefined;

    if (title || artist) {
      const by = title && artist ? `${title} - ${artist}` : (title ?? artist);
      if (by) bits.push(by);
    }

    const channels = typeof meta.chn === "number" ? meta.chn : undefined;
    if (typeof channels === "number") {
      bits.push(`${channels} ch`);
    }

    this.setInfoText(bits.join(" | "));
  }

  private describeReadyText(): string {
    const parts: string[] = ["Music preview"];
    if (this.isDirty) parts.push("out of date");
    if (this.currentMimeType) parts.push(this.currentMimeType);
    if (this.filename) parts.push(this.filename);
    return parts.join(" | ");
  }

  private applyMusicInspectorStateFromStorage(): void {
    const filename = this.filename ?? "editor.star";

    const fallback: MusicInspectorStateV1 = {
      v: 1,
      loopMode: "song",
      loopStartOrder: 0,
      loopEndOrder: 0,
    };

    const state = this.coerceMusicInspectorState(
      loadInspectSection<MusicInspectorStateV1>(filename, "music", fallback),
      fallback
    );

    this.loopMode = state.loopMode;
    this.loopStartOrder = state.loopStartOrder;
    this.loopEndOrder = state.loopEndOrder;
    this.updateLoopUI();
  }

  private coerceMusicInspectorState(
    input: MusicInspectorStateV1,
    fallback: MusicInspectorStateV1
  ): MusicInspectorStateV1 {
    if (!input || input.v !== 1) return fallback;

    const loopMode: LoopMode =
      input.loopMode === "song" || input.loopMode === "order" || input.loopMode === "order_range" || input.loopMode === "off"
        ? input.loopMode
        : fallback.loopMode;

    const loopStartOrder =
      typeof input.loopStartOrder === "number" && Number.isFinite(input.loopStartOrder)
        ? Math.max(0, Math.trunc(input.loopStartOrder))
        : fallback.loopStartOrder;
    const loopEndOrder =
      typeof input.loopEndOrder === "number" && Number.isFinite(input.loopEndOrder)
        ? Math.max(0, Math.trunc(input.loopEndOrder))
        : fallback.loopEndOrder;

    return {
      v: 1,
      loopMode,
      loopStartOrder,
      loopEndOrder,
    };
  }

  private persistMusicInspectorState(): void {
    const filename = this.filename ?? "editor.star";
    const state: MusicInspectorStateV1 = {
      v: 1,
      loopMode: this.loopMode,
      loopStartOrder: this.loopStartOrder,
      loopEndOrder: this.loopEndOrder,
    };
    saveInspectSection(filename, "music", state);
  }

  private getCurrentOrderForLoop(): number {
    const ord = this.nowPlaying.order;
    const n = typeof ord === "number" && Number.isFinite(ord) ? Math.trunc(ord) : 0;
    return Math.max(0, n);
  }

  private setLoopRange(range: { start: number; end: number }): void {
    let start = Number.isFinite(range.start) ? Math.max(0, Math.trunc(range.start)) : 0;
    let end = Number.isFinite(range.end) ? Math.max(0, Math.trunc(range.end)) : start;

    const total = this.totalOrders;
    if (typeof total === "number" && Number.isFinite(total) && total > 0) {
      const clamped = clampOrderRange({ start, end }, total);
      start = clamped.start;
      end = clamped.end;
    } else if (end < start) {
      end = start;
    }

    if (this.loopMode === "order") {
      end = start;
    }

    this.loopStartOrder = start;
    this.loopEndOrder = end;
    this.updateLoopUI();
    this.persistMusicInspectorState();
    this.updateOrderChipHighlights();
  }

  private setLoopStartToCurrent(): void {
    const cur = this.getCurrentOrderForLoop();
    if (this.loopMode === "order") {
      this.setLoopRange({ start: cur, end: cur });
      return;
    }
    const end = Math.max(cur, this.loopEndOrder);
    this.setLoopRange({ start: cur, end });
  }

  private setLoopEndToCurrent(): void {
    const cur = this.getCurrentOrderForLoop();
    if (this.loopMode === "order") {
      this.setLoopRange({ start: cur, end: cur });
      return;
    }
    const start = Math.min(cur, this.loopStartOrder);
    this.setLoopRange({ start, end: cur });
  }

  private setLoopRangeToCurrent(): void {
    const cur = this.getCurrentOrderForLoop();
    this.setLoopRange({ start: cur, end: cur });
  }

  private setLoopRangeToFull(): void {
    const total = this.totalOrders;
    const end = typeof total === "number" && Number.isFinite(total) && total > 0 ? Math.max(0, total - 1) : 0;
    this.setLoopRange({ start: 0, end });
  }

  private toggleLoopModeSongRange(): void {
    this.loopMode = this.loopMode === "order_range" ? "song" : "order_range";

    if (
      this.loopMode === "order_range" &&
      this.loopStartOrder === 0 &&
      this.loopEndOrder === 0 &&
      typeof this.totalOrders === "number" &&
      Number.isFinite(this.totalOrders) &&
      this.totalOrders > 0
    ) {
      this.loopEndOrder = Math.max(0, this.totalOrders - 1);
    }

    this.clampLoopOrders();
    this.updateLoopUI();
    this.persistMusicInspectorState();
    this.applyLoopModeToPlayer();
    this.updateOrderChipHighlights();
  }

  private onKeyDown(e: KeyboardEvent): void {
    if (e.ctrlKey || e.metaKey || e.altKey) return;

    const target = e.target as HTMLElement | null;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLSelectElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLButtonElement ||
      (target?.isContentEditable ?? false)
    ) {
      return;
    }

    if (e.key === "[") {
      e.preventDefault();
      e.stopPropagation();
      this.setLoopStartToCurrent();
      return;
    }

    if (e.key === "]") {
      e.preventDefault();
      e.stopPropagation();
      this.setLoopEndToCurrent();
      return;
    }

    if (e.key === "\\") {
      e.preventDefault();
      e.stopPropagation();
      this.toggleLoopModeSongRange();
    }
  }

  private updateLoopUI(): void {
    this.loopModeSelect.value = this.loopMode;
    this.updateLoopSelectVisibility();
    this.updateLoopOrderOptions();
  }

  private updateLoopSelectVisibility(): void {
    const showRange = this.loopMode === "order" || this.loopMode === "order_range";
    this.loopStartSelect.style.display = showRange ? "inline-block" : "none";
    this.loopEndSelect.style.display = this.loopMode === "order_range" ? "inline-block" : "none";
  }

  private clampLoopOrders(): void {
    const total = this.totalOrders;
    if (typeof total !== "number" || !Number.isFinite(total) || total <= 0) return;

    const inputEnd = this.loopMode === "order" ? this.loopStartOrder : this.loopEndOrder;
    const clamped = clampOrderRange({ start: this.loopStartOrder, end: inputEnd }, total);
    this.loopStartOrder = clamped.start;
    this.loopEndOrder = this.loopMode === "order" ? clamped.start : clamped.end;
  }

  private updateLoopOrderOptions(): void {
    const total = this.totalOrders;
    const hasTotal = typeof total === "number" && Number.isFinite(total) && total > 0;

    const replaceOptions = (
      select: HTMLSelectElement,
      opts: Array<{ label: string; value: string }>,
      value: number,
      enabled: boolean
    ) => {
      while (select.firstChild) select.removeChild(select.firstChild);
      for (const optInfo of opts) {
        const opt = document.createElement("option");
        opt.value = optInfo.value;
        opt.textContent = optInfo.label;
        select.appendChild(opt);
      }
      select.value = String(value);
      select.disabled = !enabled;
    };

    if (!hasTotal) {
      const placeholder = [{ label: "O--", value: "0" }];
      replaceOptions(this.loopStartSelect, placeholder, 0, false);
      replaceOptions(this.loopEndSelect, placeholder, 0, false);
      return;
    }

    const maxOrder = total - 1;

    let start = Math.max(0, Math.trunc(this.loopStartOrder));
    let end = Math.max(0, Math.trunc(this.loopEndOrder));
    if (this.loopMode === "order") end = start;

    const clamped = clampOrderRange({ start, end }, total);
    start = clamped.start;
    end = this.loopMode === "order" ? clamped.start : clamped.end;

    this.loopStartOrder = start;
    this.loopEndOrder = end;

    const options: Array<{ label: string; value: string }> = [];
    for (let i = 0; i <= maxOrder; i++) {
      options.push({ label: `O${this.format2(i)}`, value: String(i) });
    }

    replaceOptions(this.loopStartSelect, options, this.loopStartOrder, true);
    replaceOptions(this.loopEndSelect, options, this.loopEndOrder, true);
  }

  private applyLoopModeToPlayer(): void {
    try {
      if (this.loopMode === "off") {
        this.player.setRepeatCount(0);
      } else {
        this.player.setRepeatCount(-1);
      }
    } catch {
      // ignore
    }
  }

  private enforceLoop(order: number): void {
    if (this.isSeeking) return;
    if (this.loopMode === "song" || this.loopMode === "off") return;
    if (this.totalOrders === null) return;

    this.clampLoopOrders();
    const start = this.loopStartOrder;
    const end = this.loopMode === "order" ? this.loopStartOrder : this.loopEndOrder;
    if (order >= start && order <= end) return;

    const now = performance.now();
    if (now - this.lastLoopSeekAtMs < 250) return;
    this.lastLoopSeekAtMs = now;

    try {
      this.player.setOrderRow(start, 0);
      return;
    } catch {
      // ignore
    }

    const knownStart = this.orderStartSecByOrder.get(start);
    const target = typeof knownStart === "number" && Number.isFinite(knownStart) ? knownStart : 0;

    try {
      this.player.setPos(target);
    } catch {
      // ignore
    }

    this.pendingSeekOrder = null;

    if (!this.isSeeking) {
      this.seekSlider.value = String(target);
      this.updateTimeText(target, this.durationSec ?? 0);
    }
  }

  private applyPendingSeek(): void {
    if (this.pendingSeekOrder === null) return;
    const order = this.pendingSeekOrder;
    this.pendingSeekOrder = null;

    try {
      this.player.setOrderRow(order, 0);
      return;
    } catch {
      // ignore
    }

    const knownStart = this.orderStartSecByOrder.get(order);
    if (typeof knownStart === "number" && Number.isFinite(knownStart)) {
      try {
        this.player.setPos(knownStart);
      } catch {
        // ignore
      }
    }
  }

  private async startPlaybackFromCurrentBuffer(opts?: { keepPaused?: boolean; restart?: boolean }): Promise<void> {
    const buf = this.currentBuffer;
    if (!buf) return;
    if (this.isPlaying && opts?.restart !== true) return;
    if (this.isPlaying && opts?.restart === true) {
      this.stop();
    }

    await this.playerReady;

    const ok = await this.ensureAudioContextRunning();
    if (!ok) {
      this.isPlaying = false;
      this.isPaused = false;
      this.playButton.textContent = "Play";
      this.setInfoText(
        "Audio is blocked by the browser. Click inside the page, then press Play again."
      );
      return;
    }

    this.applyLoopModeToPlayer();
    this.player.play(buf);
    this.applyPendingSeek();
    this.isPlaying = true;
    this.isPaused = false;
    this.playButton.textContent = "Pause";

    if (opts?.keepPaused === true) {
      this.player.pause();
      this.isPaused = true;
      this.playButton.textContent = "Play";
    }
  }

  private async ensureAudioContextRunning(): Promise<boolean> {
    // Some browsers create the AudioContext in a suspended state until a
    // user gesture occurs.
    try {
      const ctx = this.player.context;
      if (ctx.state === "running") return true;
      if (ctx.state === "closed") return false;
      if (ctx.state === "suspended") {
        await ctx.resume();
      }
      // Re-read through player to avoid TS narrowing on ctx.state.
      return this.player.context.state === "running";
    } catch {
      return false;
    }
  }

  private markDirty(): void {
    if (!this.currentBuffer) return;
    this.isDirty = this.currentBufferKey !== this.getDesiredSourceKey();
  }

  private getDesiredSourceKey(): string | null {
    if (!this.source || !this.filename) return null;
    return this.makeSourceKey(this.source, this.filename);
  }

  private makeSourceKey(source: string, filename: string): string {
    return `${filename}\n${source}`;
  }

  private updateTimeText(currentSec: number, totalSec: number): void {
    this.currentTimeText.textContent = this.formatTime(currentSec);
    if (totalSec > 0) this.totalTimeText.textContent = this.formatTime(totalSec);
  }

  private formatTime(sec: number): string {
    if (!Number.isFinite(sec) || sec < 0) return "0:00";
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${String(s).padStart(2, "0")}`;
  }

  private decodeBase64(base64: string): ArrayBuffer {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
  }

  private describeError(err: unknown): string {
    if (err instanceof Error) return err.message;
    if (typeof err === "string") return err;
    try {
      return JSON.stringify(err);
    } catch {
      return "Unknown error";
    }
  }

  private loadLivePreviewEnabled(): boolean {
    try {
      const raw = localStorage.getItem(LIVE_PREVIEW_STORAGE_KEY);
      if (raw === null) return false;
      return raw === "1";
    } catch {
      return false;
    }
  }

  private storeLivePreviewEnabled(enabled: boolean): void {
    try {
      localStorage.setItem(LIVE_PREVIEW_STORAGE_KEY, enabled ? "1" : "0");
    } catch {
      // ignore
    }
  }
}
