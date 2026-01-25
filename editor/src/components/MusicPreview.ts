import { ChiptuneJsPlayer } from "chiptune3";

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
      const sec = Number(this.seekSlider.value);
      this.updateTimeText(sec, this.durationSec ?? 0);
      this.player.setPos(sec);
    });
    seekRow.appendChild(this.seekSlider);

    const seekRight = document.createElement("div");
    seekRight.textContent = "0:00";
    seekRight.id = "music-total";
    seekRight.style.textAlign = "left";
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
        const total = this.wrapper.querySelector("#music-total");
        if (total) total.textContent = this.formatTime(dur);
      }
      this.updateInfoText(meta);
    });

    this.player.onProgress((e) => {
      if (this.isDisposed) return;
      if (!this.isPlaying) return;
      const pos = typeof e.pos === "number" ? e.pos : (this.player.getCurrentTime() ?? 0);
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
    });

    this.player.onError((err) => {
      if (this.isDisposed) return;
      this.setInfoText(`Music preview error: ${this.describeError(err)}`);
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

    this.player.play(buf);
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
    const current = this.wrapper.querySelector("#music-current");
    if (current) current.textContent = this.formatTime(currentSec);
    const total = this.wrapper.querySelector("#music-total");
    if (total && totalSec > 0) total.textContent = this.formatTime(totalSec);
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
