import { loadInspectSection, saveInspectSection } from "../lib/storage";

type AudioInspectorStateV1 = {
  v: 1;
  loopStartSec: number;
  loopEndSec: number;
  normalize: boolean;
  spectrumVisible: boolean;
};

type AudioStats = {
  peak: number;
  peakDb: number;
  rms: number;
  rmsDb: number;
  clipCount: number;
  dcOffset: number;
};

/**
 * Audio preview component with waveform + basic inspection.
 */
export class AudioPreview {
  private container: HTMLElement;

  private waveformCanvas: HTMLCanvasElement;
  private waveformCtx: CanvasRenderingContext2D;
  private overlayCanvas: HTMLCanvasElement;
  private overlayCtx: CanvasRenderingContext2D;
  private spectrumCanvas: HTMLCanvasElement;
  private spectrumCtx: CanvasRenderingContext2D;

  private playButton: HTMLButtonElement;
  private stopButton: HTMLButtonElement;
  private progressBar: HTMLDivElement;
  private statsDiv: HTMLDivElement;

  private loopLabel: HTMLDivElement;
  private loopStartInput: HTMLInputElement;
  private loopEndInput: HTMLInputElement;
  private normalizeInput: HTMLInputElement;
  private spectrumInput: HTMLInputElement;

  private audioContext: AudioContext | null = null;
  private audioBuffer: AudioBuffer | null = null;
  private sourceNode: AudioBufferSourceNode | null = null;
  private analyserNode: AnalyserNode | null = null;
  private gainNode: GainNode | null = null;

  private isPlaying = false;
  private animationId: number | null = null;
  private startTime = 0;
  private pauseTime = 0;

  private filePathForState: string = "editor.star";
  private inspectorState: AudioInspectorStateV1 = {
    v: 1,
    loopStartSec: 0,
    loopEndSec: 0,
    normalize: false,
    spectrumVisible: true,
  };

  private stats: AudioStats | null = null;

  private dpr = 1;
  private waveformCssW = 0;
  private waveformCssH = 0;

  constructor(container: HTMLElement) {
    this.container = container;

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

    const waveWrap = document.createElement("div");
    waveWrap.style.cssText = `
      position: relative;
      flex: 1;
      width: 100%;
      background: #1e1e1e;
      border-radius: 4px;
      overflow: hidden;
    `;

    this.waveformCanvas = document.createElement("canvas");
    this.waveformCanvas.style.cssText = `
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
    `;
    waveWrap.appendChild(this.waveformCanvas);
    this.waveformCtx = this.waveformCanvas.getContext("2d")!;

    this.overlayCanvas = document.createElement("canvas");
    this.overlayCanvas.style.cssText = `
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      pointer-events: none;
    `;
    waveWrap.appendChild(this.overlayCanvas);
    this.overlayCtx = this.overlayCanvas.getContext("2d")!;

    wrapper.appendChild(waveWrap);

    this.spectrumCanvas = document.createElement("canvas");
    this.spectrumCanvas.style.cssText = `
      width: 100%;
      height: 72px;
      background: #151515;
      border-radius: 4px;
      display: block;
    `;
    wrapper.appendChild(this.spectrumCanvas);
    this.spectrumCtx = this.spectrumCanvas.getContext("2d")!;

    // Progress bar
    this.progressBar = document.createElement("div");
    this.progressBar.style.cssText = `
      width: 100%;
      height: 4px;
      background: #333;
      border-radius: 2px;
      overflow: hidden;
    `;
    const progressFill = document.createElement("div");
    progressFill.style.cssText = `
      height: 100%;
      width: 0%;
      background: #007acc;
      transition: width 0.05s linear;
    `;
    progressFill.id = "progress-fill";
    this.progressBar.appendChild(progressFill);
    wrapper.appendChild(this.progressBar);

    // Controls
    const controls = document.createElement("div");
    controls.style.cssText = `
      display: flex;
      gap: 8px;
      justify-content: center;
    `;

    this.playButton = document.createElement("button");
    this.playButton.type = "button";
    this.playButton.textContent = "Play";
    this.playButton.style.cssText = `
      padding: 6px 16px;
      background: #007acc;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
    `;
    this.playButton.onclick = () => this.togglePlay();
    controls.appendChild(this.playButton);

    this.stopButton = document.createElement("button");
    this.stopButton.type = "button";
    this.stopButton.textContent = "Stop";
    this.stopButton.style.cssText = `
      padding: 6px 16px;
      background: #444;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
    `;
    this.stopButton.onclick = () => this.stop();
    controls.appendChild(this.stopButton);

    wrapper.appendChild(controls);

    // Stats + inspector
    const inspector = document.createElement("div");
    inspector.style.cssText = `
      display: flex;
      flex-direction: column;
      gap: 6px;
      padding: 6px 0;
      font-size: 11px;
      color: #bbb;
      user-select: text;
    `;

    this.statsDiv = document.createElement("div");
    this.statsDiv.style.cssText = `
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      justify-content: center;
      color: #bdbdbd;
    `;
    inspector.appendChild(this.statsDiv);

    this.loopLabel = document.createElement("div");
    this.loopLabel.style.cssText = `
      text-align: center;
      color: #bdbdbd;
    `;
    this.loopLabel.textContent = "Loop: 0.000-0.000s";
    inspector.appendChild(this.loopLabel);

    const loopSliders = document.createElement("div");
    loopSliders.style.cssText = `
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
      align-items: center;
    `;

    this.loopStartInput = document.createElement("input");
    this.loopStartInput.type = "range";
    this.loopStartInput.min = "0";
    this.loopStartInput.step = "0.001";
    this.loopStartInput.value = "0";
    loopSliders.appendChild(this.loopStartInput);

    this.loopEndInput = document.createElement("input");
    this.loopEndInput.type = "range";
    this.loopEndInput.min = "0";
    this.loopEndInput.step = "0.001";
    this.loopEndInput.value = "0";
    loopSliders.appendChild(this.loopEndInput);
    inspector.appendChild(loopSliders);

    const toggles = document.createElement("div");
    toggles.style.cssText = `
      display: flex;
      gap: 14px;
      justify-content: center;
      flex-wrap: wrap;
      align-items: center;
      color: #c7c7c7;
    `;

    this.normalizeInput = document.createElement("input");
    this.normalizeInput.type = "checkbox";
    const normalizeLabel = document.createElement("label");
    normalizeLabel.style.cssText = `display:flex;gap:6px;align-items:center; user-select:none;`;
    normalizeLabel.appendChild(this.normalizeInput);
    normalizeLabel.appendChild(document.createTextNode("Normalize (-1 dBFS)"));
    toggles.appendChild(normalizeLabel);

    this.spectrumInput = document.createElement("input");
    this.spectrumInput.type = "checkbox";
    const spectrumLabel = document.createElement("label");
    spectrumLabel.style.cssText = `display:flex;gap:6px;align-items:center; user-select:none;`;
    spectrumLabel.appendChild(this.spectrumInput);
    spectrumLabel.appendChild(document.createTextNode("Spectrum"));
    toggles.appendChild(spectrumLabel);

    inspector.appendChild(toggles);
    wrapper.appendChild(inspector);

    // Duration display
    const durationDiv = document.createElement("div");
    durationDiv.id = "duration-display";
    durationDiv.style.cssText = `
      font-size: 11px;
      color: #999;
      text-align: center;
    `;
    durationDiv.textContent = "0:00 / 0:00 • Preview";
    wrapper.appendChild(durationDiv);

    container.appendChild(wrapper);

    // Handlers
    const applyLoop = () => {
      if (!this.audioBuffer) return;
      const dur = this.audioBuffer.duration;
      let start = Number(this.loopStartInput.value);
      let end = Number(this.loopEndInput.value);
      if (!Number.isFinite(start)) start = 0;
      if (!Number.isFinite(end)) end = dur;
      start = Math.max(0, Math.min(dur, start));
      end = Math.max(0, Math.min(dur, end));
      if (end <= start) end = Math.min(dur, start + 0.01);

      this.inspectorState.loopStartSec = start;
      this.inspectorState.loopEndSec = end;
      this.loopStartInput.value = String(start);
      this.loopEndInput.value = String(end);
      this.updateLoopLabel();
      this.persistInspectorState();

      if (this.isPlaying) {
        const pos = this.getPlaybackPositionSec();
        this.pauseTime = Math.max(start, Math.min(end, pos));
        this.restartPlayback();
      } else {
        this.drawLoopOverlay();
      }
    };

    this.loopStartInput.addEventListener("input", applyLoop);
    this.loopEndInput.addEventListener("input", applyLoop);

    this.normalizeInput.addEventListener("change", () => {
      this.inspectorState.normalize = this.normalizeInput.checked;
      this.persistInspectorState();
      this.applyNormalizeGain();
    });

    this.spectrumInput.addEventListener("change", () => {
      this.inspectorState.spectrumVisible = this.spectrumInput.checked;
      this.spectrumCanvas.style.display = this.inspectorState.spectrumVisible ? "block" : "none";
      this.persistInspectorState();
    });

    // Resize
    const resizeObserver = new ResizeObserver(() => this.handleResize());
    resizeObserver.observe(container);
    this.handleResize();
    this.drawEmptyWaveform();
  }

  async loadWAV(base64Data: string, filePath: string = "editor.star"): Promise<void> {
    this.filePathForState = filePath || "editor.star";

    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }

    const binaryString = atob(base64Data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }

    try {
      this.stop();
      this.audioBuffer = await this.audioContext.decodeAudioData(bytes.buffer);

      const dur = this.audioBuffer.duration;

      const fallback: AudioInspectorStateV1 = {
        v: 1,
        loopStartSec: 0,
        loopEndSec: dur,
        normalize: false,
        spectrumVisible: true,
      };
      const state = this.coerceInspectorState(
        loadInspectSection<AudioInspectorStateV1>(this.filePathForState, "audio", fallback),
        fallback,
        dur
      );
      this.inspectorState = state;

      this.loopStartInput.max = String(dur);
      this.loopEndInput.max = String(dur);
      this.loopStartInput.value = String(state.loopStartSec);
      this.loopEndInput.value = String(state.loopEndSec);
      this.normalizeInput.checked = state.normalize;
      this.spectrumInput.checked = state.spectrumVisible;
      this.spectrumCanvas.style.display = state.spectrumVisible ? "block" : "none";

      this.stats = this.computeStats(this.audioBuffer);
      this.renderStats();

      this.drawWaveform();
      this.drawLoopOverlay();
      this.updateLoopLabel();
      this.updateDuration(0, dur);
      this.updateSampleInfo(this.audioBuffer.sampleRate, this.audioBuffer.numberOfChannels);
    } catch (e) {
      console.error("Failed to decode audio:", e);
      throw new Error(`Failed to decode audio: ${e}`);
    }
  }

  clear(): void {
    this.stop();
    this.audioBuffer = null;
    this.stats = null;
    this.statsDiv.textContent = "";
    this.drawEmptyWaveform();
  }

  dispose(): void {
    this.stop();
    if (this.audioContext) {
      this.audioContext.close();
    }
    this.container.innerHTML = "";
  }

  private coerceInspectorState(
    input: AudioInspectorStateV1,
    fallback: AudioInspectorStateV1,
    durationSec: number
  ): AudioInspectorStateV1 {
    if (!input || input.v !== 1) return fallback;

    const loopStartSecRaw = typeof input.loopStartSec === "number" ? input.loopStartSec : fallback.loopStartSec;
    const loopEndSecRaw = typeof input.loopEndSec === "number" ? input.loopEndSec : fallback.loopEndSec;
    const normalize = typeof input.normalize === "boolean" ? input.normalize : fallback.normalize;
    const spectrumVisible =
      typeof input.spectrumVisible === "boolean" ? input.spectrumVisible : fallback.spectrumVisible;

    let loopStartSec = Math.max(0, Math.min(durationSec, loopStartSecRaw));
    let loopEndSec = Math.max(0, Math.min(durationSec, loopEndSecRaw));
    if (loopEndSec <= loopStartSec) loopEndSec = Math.min(durationSec, loopStartSec + 0.01);

    return { v: 1, loopStartSec, loopEndSec, normalize, spectrumVisible };
  }

  private persistInspectorState(): void {
    saveInspectSection(this.filePathForState, "audio", this.inspectorState);
  }

  private computeStats(buffer: AudioBuffer): AudioStats {
    const clipThreshold = 0.999;
    let peak = 0;
    let sumSq = 0;
    let sum = 0;
    let n = 0;
    let clipCount = 0;

    for (let ch = 0; ch < buffer.numberOfChannels; ch++) {
      const data = buffer.getChannelData(ch);
      for (let i = 0; i < data.length; i++) {
        const x = data[i];
        const ax = Math.abs(x);
        if (ax > peak) peak = ax;
        if (ax >= clipThreshold) clipCount++;
        sumSq += x * x;
        sum += x;
        n++;
      }
    }

    const rms = n > 0 ? Math.sqrt(sumSq / n) : 0;
    const dcOffset = n > 0 ? sum / n : 0;
    return {
      peak,
      peakDb: this.linearToDbFS(peak),
      rms,
      rmsDb: this.linearToDbFS(rms),
      clipCount,
      dcOffset,
    };
  }

  private linearToDbFS(x: number): number {
    const v = Math.max(1e-12, x);
    return 20 * Math.log10(v);
  }

  private renderStats(): void {
    if (!this.audioBuffer || !this.stats) {
      this.statsDiv.textContent = "";
      return;
    }

    const ch = this.audioBuffer.numberOfChannels;
    const channelStr = ch === 1 ? "mono" : ch === 2 ? "stereo" : `${ch}ch`;
    const clipBadge =
      this.stats.clipCount > 0
        ? `<span style="color:#fff;background:#b00020;padding:1px 6px;border-radius:10px;">CLIP</span>`
        : "";

    this.statsDiv.innerHTML = `
      <span>${this.audioBuffer.duration.toFixed(3)}s</span>
      <span>${this.audioBuffer.sampleRate}Hz ${channelStr}</span>
      <span>Peak ${this.stats.peak.toFixed(3)} (${this.stats.peakDb.toFixed(1)} dBFS)</span>
      <span>RMS ${this.stats.rms.toFixed(3)} (${this.stats.rmsDb.toFixed(1)} dBFS)</span>
      <span>DC ${this.stats.dcOffset.toFixed(5)}</span>
      <span>Clip ${this.stats.clipCount}</span>
      ${clipBadge}
    `;
  }

  private updateLoopLabel(): void {
    this.loopLabel.textContent = `Loop: ${this.inspectorState.loopStartSec.toFixed(3)}-${this.inspectorState.loopEndSec.toFixed(
      3
    )}s`;
  }

  private togglePlay(): void {
    if (!this.audioContext || !this.audioBuffer) return;

    if (this.isPlaying) {
      this.pause();
      return;
    }

    if (this.audioContext.state === "suspended") {
      this.audioContext.resume();
    }

    // Clamp resume to loop.
    this.pauseTime = this.clampToLoop(this.pauseTime);
    this.startPlayback();
  }

  private pause(): void {
    if (!this.audioContext || !this.audioBuffer) return;

    this.pauseTime = this.getPlaybackPositionSec();
    if (this.sourceNode) {
      try {
        this.sourceNode.stop();
      } catch {
        // ignore
      }
      this.sourceNode = null;
    }

    this.isPlaying = false;
    this.playButton.textContent = "Play";
    if (this.animationId) cancelAnimationFrame(this.animationId);
    this.animationId = null;
    this.drawLoopOverlay();
  }

  private stop(): void {
    if (this.sourceNode) {
      try {
        this.sourceNode.stop();
      } catch {
        // ignore
      }
      this.sourceNode = null;
    }

    this.isPlaying = false;
    this.pauseTime = 0;
    this.playButton.textContent = "Play";
    this.updateProgress(0);
    if (this.audioBuffer) this.updateDuration(0, this.audioBuffer.duration);
    if (this.animationId) cancelAnimationFrame(this.animationId);
    this.animationId = null;
    if (this.audioBuffer) this.drawWaveform();
    else this.drawEmptyWaveform();
    this.drawLoopOverlay();
  }

  private restartPlayback(): void {
    if (!this.audioContext || !this.audioBuffer) return;
    if (this.sourceNode) {
      try {
        this.sourceNode.stop();
      } catch {
        // ignore
      }
      this.sourceNode = null;
    }
    this.startPlayback();
  }

  private startPlayback(): void {
    if (!this.audioContext || !this.audioBuffer) return;

    const source = this.audioContext.createBufferSource();
    source.buffer = this.audioBuffer;

    // Apply loop region.
    source.loop = true;
    source.loopStart = this.inspectorState.loopStartSec;
    source.loopEnd = this.inspectorState.loopEndSec;

    this.gainNode = this.audioContext.createGain();
    this.analyserNode = this.audioContext.createAnalyser();
    this.analyserNode.fftSize = 2048;

    source.connect(this.gainNode);
    this.gainNode.connect(this.analyserNode);
    this.analyserNode.connect(this.audioContext.destination);

    this.sourceNode = source;
    this.applyNormalizeGain();

    source.onended = () => {
      // When paused/stopped we stop the node manually; ignore.
      if (!this.isPlaying) return;
      this.isPlaying = false;
      this.playButton.textContent = "Play";
      this.updateProgress(0);
      if (this.animationId) cancelAnimationFrame(this.animationId);
      this.animationId = null;
    };

    this.startTime = this.audioContext.currentTime - this.pauseTime;
    source.start(0, this.pauseTime);

    this.isPlaying = true;
    this.playButton.textContent = "Pause";
    this.visualize();
  }

  private applyNormalizeGain(): void {
    if (!this.gainNode || !this.stats) return;
    if (!this.inspectorState.normalize) {
      this.gainNode.gain.value = 1;
      return;
    }

    const targetPeak = Math.pow(10, -1 / 20); // -1 dBFS
    const peak = Math.max(1e-9, this.stats.peak);
    this.gainNode.gain.value = targetPeak / peak;
  }

  private clampToLoop(t: number): number {
    if (!this.audioBuffer) return 0;
    const dur = this.audioBuffer.duration;
    const a = Math.max(0, Math.min(dur, this.inspectorState.loopStartSec));
    const b = Math.max(0, Math.min(dur, this.inspectorState.loopEndSec));
    const end = Math.max(a + 0.01, b);
    return Math.max(a, Math.min(end, t));
  }

  private getPlaybackPositionSec(): number {
    if (!this.audioContext || !this.audioBuffer) return this.pauseTime;

    const t = this.audioContext.currentTime - this.startTime;
    const dur = this.audioBuffer.duration;
    const loopStart = this.inspectorState.loopStartSec;
    const loopEnd = this.inspectorState.loopEndSec;

    // If no valid loop region, clamp.
    if (!(loopEnd > loopStart)) return Math.max(0, Math.min(dur, t));

    // Convert time to a looping position.
    if (t < loopEnd) return Math.max(0, Math.min(dur, t));
    const span = loopEnd - loopStart;
    const wrapped = loopStart + ((t - loopStart) % span);
    return Math.max(0, Math.min(dur, wrapped));
  }

  private drawWaveform(): void {
    if (!this.audioBuffer) {
      this.drawEmptyWaveform();
      return;
    }

    const data = this.audioBuffer.getChannelData(0);
    const width = this.waveformCssW;
    const height = this.waveformCssH;

    this.waveformCtx.clearRect(0, 0, width, height);
    this.waveformCtx.fillStyle = "#1e1e1e";
    this.waveformCtx.fillRect(0, 0, width, height);

    this.waveformCtx.beginPath();
    this.waveformCtx.strokeStyle = "#4a9eff";
    this.waveformCtx.lineWidth = 1;

    const step = Math.ceil(data.length / Math.max(1, width));
    const amp = height / 2;

    for (let i = 0; i < width; i++) {
      let min = 1.0;
      let max = -1.0;
      for (let j = 0; j < step; j++) {
        const datum = data[i * step + j];
        if (datum < min) min = datum;
        if (datum > max) max = datum;
      }
      this.waveformCtx.moveTo(i, (1 + min) * amp);
      this.waveformCtx.lineTo(i, (1 + max) * amp);
    }

    this.waveformCtx.stroke();
  }

  private drawEmptyWaveform(): void {
    const width = this.waveformCssW;
    const height = this.waveformCssH;

    this.waveformCtx.clearRect(0, 0, width, height);
    this.waveformCtx.fillStyle = "#1e1e1e";
    this.waveformCtx.fillRect(0, 0, width, height);

    this.waveformCtx.fillStyle = "#666";
    this.waveformCtx.font = "12px sans-serif";
    this.waveformCtx.textAlign = "center";
    this.waveformCtx.fillText("No audio loaded", width / 2, height / 2);

    this.overlayCtx.clearRect(0, 0, width, height);
  }

  private drawLoopOverlay(playheadPos?: number): void {
    const w = this.waveformCssW;
    const h = this.waveformCssH;
    this.overlayCtx.clearRect(0, 0, w, h);
    if (!this.audioBuffer) return;

    const dur = this.audioBuffer.duration;
    if (dur <= 0) return;

    const loopStartX = Math.floor((this.inspectorState.loopStartSec / dur) * w);
    const loopEndX = Math.floor((this.inspectorState.loopEndSec / dur) * w);

    this.overlayCtx.save();

    // Shade outside loop region.
    this.overlayCtx.fillStyle = "rgba(0, 0, 0, 0.25)";
    this.overlayCtx.fillRect(0, 0, Math.max(0, loopStartX), h);
    this.overlayCtx.fillRect(Math.max(0, loopEndX), 0, Math.max(0, w - loopEndX), h);

    // Loop markers.
    this.overlayCtx.strokeStyle = "rgba(255, 255, 255, 0.35)";
    this.overlayCtx.lineWidth = 1;
    this.overlayCtx.beginPath();
    this.overlayCtx.moveTo(loopStartX + 0.5, 0);
    this.overlayCtx.lineTo(loopStartX + 0.5, h);
    this.overlayCtx.moveTo(loopEndX + 0.5, 0);
    this.overlayCtx.lineTo(loopEndX + 0.5, h);
    this.overlayCtx.stroke();

    // Playhead.
    if (typeof playheadPos === "number" && Number.isFinite(playheadPos)) {
      const x = Math.floor((playheadPos / dur) * w);
      this.overlayCtx.strokeStyle = "rgba(255, 204, 0, 0.9)";
      this.overlayCtx.beginPath();
      this.overlayCtx.moveTo(x + 0.5, 0);
      this.overlayCtx.lineTo(x + 0.5, h);
      this.overlayCtx.stroke();
    }

    this.overlayCtx.restore();
  }

  private visualize(): void {
    if (!this.isPlaying || !this.analyserNode || !this.audioContext || !this.audioBuffer) return;
    this.animationId = requestAnimationFrame(() => this.visualize());

    const pos = this.getPlaybackPositionSec();
    const dur = this.audioBuffer.duration;

    this.updateProgress(Math.min(pos / dur, 1));
    this.updateDuration(pos, dur);
    this.drawLoopOverlay(pos);

    if (!this.inspectorState.spectrumVisible) return;

    const dataArray = new Uint8Array(this.analyserNode.frequencyBinCount);
    this.analyserNode.getByteFrequencyData(dataArray);

    const sw = this.spectrumCanvas.width / this.dpr;
    const sh = this.spectrumCanvas.height / this.dpr;

    this.spectrumCtx.clearRect(0, 0, sw, sh);
    this.spectrumCtx.fillStyle = "#151515";
    this.spectrumCtx.fillRect(0, 0, sw, sh);

    const barW = Math.max(1, Math.floor(sw / dataArray.length));
    for (let i = 0; i < dataArray.length; i++) {
      const v = dataArray[i] / 255;
      const bh = Math.floor(v * sh);
      this.spectrumCtx.fillStyle = "#4a9eff";
      this.spectrumCtx.fillRect(i * barW, sh - bh, barW, bh);
    }
  }

  private updateProgress(progress: number): void {
    const fill = this.progressBar.querySelector("#progress-fill") as HTMLDivElement;
    if (fill) {
      fill.style.width = `${progress * 100}%`;
    }
  }

  private updateDuration(current: number, total: number): void {
    const formatTime = (s: number) => {
      const mins = Math.floor(s / 60);
      const secs = Math.floor(s % 60);
      return `${mins}:${secs.toString().padStart(2, "0")}`;
    };

    const display = this.container.querySelector("#duration-display");
    if (display) {
      display.textContent = `${formatTime(current)} / ${formatTime(total)}`;
    }
  }

  private updateSampleInfo(sampleRate: number, channels: number): void {
    const display = this.container.querySelector("#duration-display");
    if (display) {
      const current = display.textContent?.split(" • ")[0] || "0:00 / 0:00";
      const channelStr = channels === 1 ? "mono" : channels === 2 ? "stereo" : `${channels}ch`;
      display.textContent = `${current} • ${sampleRate}Hz ${channelStr} • Preview`;
    }
  }

  private handleResize(): void {
    this.dpr = window.devicePixelRatio || 1;
    this.waveformCssW = Math.max(1, this.container.clientWidth - 24);
    this.waveformCssH = Math.max(100, this.container.clientHeight - 160);

    const wPx = Math.max(1, Math.round(this.waveformCssW * this.dpr));
    const hPx = Math.max(1, Math.round(this.waveformCssH * this.dpr));
    this.waveformCanvas.width = wPx;
    this.waveformCanvas.height = hPx;
    this.overlayCanvas.width = wPx;
    this.overlayCanvas.height = hPx;
    this.waveformCtx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.overlayCtx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);

    const spectrumPxW = Math.max(1, Math.round(this.waveformCssW * this.dpr));
    const spectrumPxH = Math.max(1, Math.round(72 * this.dpr));
    this.spectrumCanvas.width = spectrumPxW;
    this.spectrumCanvas.height = spectrumPxH;
    this.spectrumCtx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);

    if (this.audioBuffer) this.drawWaveform();
    else this.drawEmptyWaveform();
    this.drawLoopOverlay(this.isPlaying ? this.getPlaybackPositionSec() : undefined);
  }
}
