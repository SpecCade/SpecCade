/**
 * Audio preview component with waveform visualization.
 *
 * Plays WAV audio with Web Audio API and visualizes the waveform.
 */
export class AudioPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private playButton: HTMLButtonElement;
  private stopButton: HTMLButtonElement;
  private progressBar: HTMLDivElement;

  private audioContext: AudioContext | null = null;
  private audioBuffer: AudioBuffer | null = null;
  private sourceNode: AudioBufferSourceNode | null = null;
  private analyserNode: AnalyserNode | null = null;
  private isPlaying = false;
  private animationId: number | null = null;
  private startTime = 0;
  private pauseTime = 0;

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

    // Create canvas for waveform
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = `
      flex: 1;
      width: 100%;
      background: #1e1e1e;
      border-radius: 4px;
    `;
    wrapper.appendChild(this.canvas);
    this.ctx = this.canvas.getContext("2d")!;

    // Create progress bar
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

    // Create controls
    const controls = document.createElement("div");
    controls.style.cssText = `
      display: flex;
      gap: 8px;
      justify-content: center;
    `;

    this.playButton = document.createElement("button");
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
    this.playButton.onclick = () => this.play();
    controls.appendChild(this.playButton);

    this.stopButton = document.createElement("button");
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

    // Create duration display
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

    // Handle resize
    const resizeObserver = new ResizeObserver(() => this.handleResize());
    resizeObserver.observe(container);
    this.handleResize();

    // Draw empty waveform
    this.drawEmptyWaveform();
  }

  /**
   * Load WAV audio from base64 data.
   */
  async loadWAV(base64Data: string): Promise<void> {
    // Initialize audio context on first interaction
    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }

    // Decode base64
    const binaryString = atob(base64Data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }

    // Decode audio
    try {
      this.audioBuffer = await this.audioContext.decodeAudioData(bytes.buffer);
      this.drawWaveform();
      this.updateDuration(0, this.audioBuffer.duration);
      this.updateSampleInfo(this.audioBuffer.sampleRate, this.audioBuffer.numberOfChannels);
    } catch (e) {
      console.error("Failed to decode audio:", e);
      throw new Error(`Failed to decode audio: ${e}`);
    }
  }

  /**
   * Play the loaded audio.
   */
  play(): void {
    if (!this.audioContext || !this.audioBuffer || this.isPlaying) return;

    // Resume context if suspended (autoplay restrictions)
    if (this.audioContext.state === "suspended") {
      this.audioContext.resume();
    }

    // Create source
    this.sourceNode = this.audioContext.createBufferSource();
    this.sourceNode.buffer = this.audioBuffer;

    // Create analyser for visualization
    this.analyserNode = this.audioContext.createAnalyser();
    this.analyserNode.fftSize = 256;

    // Connect nodes
    this.sourceNode.connect(this.analyserNode);
    this.analyserNode.connect(this.audioContext.destination);

    // Handle playback end
    this.sourceNode.onended = () => {
      this.isPlaying = false;
      this.playButton.textContent = "Play";
      this.updateProgress(0);
      if (this.animationId) {
        cancelAnimationFrame(this.animationId);
      }
    };

    // Start playback
    this.startTime = this.audioContext.currentTime - this.pauseTime;
    this.sourceNode.start(0, this.pauseTime);
    this.isPlaying = true;
    this.playButton.textContent = "Pause";

    // Start visualization
    this.visualize();
  }

  /**
   * Stop playback.
   */
  stop(): void {
    if (this.sourceNode) {
      this.sourceNode.stop();
      this.sourceNode = null;
    }
    this.isPlaying = false;
    this.pauseTime = 0;
    this.playButton.textContent = "Play";
    this.updateProgress(0);
    if (this.audioBuffer) {
      this.updateDuration(0, this.audioBuffer.duration);
    }
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }
    this.drawWaveform();
  }

  /**
   * Draw the static waveform.
   */
  private drawWaveform(): void {
    if (!this.audioBuffer) {
      this.drawEmptyWaveform();
      return;
    }

    const data = this.audioBuffer.getChannelData(0);
    const width = this.canvas.width;
    const height = this.canvas.height;

    this.ctx.clearRect(0, 0, width, height);
    this.ctx.fillStyle = "#1e1e1e";
    this.ctx.fillRect(0, 0, width, height);

    this.ctx.beginPath();
    this.ctx.strokeStyle = "#4a9eff";
    this.ctx.lineWidth = 1;

    const step = Math.ceil(data.length / width);
    const amp = height / 2;

    for (let i = 0; i < width; i++) {
      let min = 1.0;
      let max = -1.0;
      for (let j = 0; j < step; j++) {
        const datum = data[i * step + j];
        if (datum < min) min = datum;
        if (datum > max) max = datum;
      }
      this.ctx.moveTo(i, (1 + min) * amp);
      this.ctx.lineTo(i, (1 + max) * amp);
    }

    this.ctx.stroke();
  }

  /**
   * Draw an empty waveform placeholder.
   */
  private drawEmptyWaveform(): void {
    const width = this.canvas.width;
    const height = this.canvas.height;

    this.ctx.clearRect(0, 0, width, height);
    this.ctx.fillStyle = "#1e1e1e";
    this.ctx.fillRect(0, 0, width, height);

    this.ctx.fillStyle = "#666";
    this.ctx.font = "12px sans-serif";
    this.ctx.textAlign = "center";
    this.ctx.fillText("No audio loaded", width / 2, height / 2);
  }

  /**
   * Visualize audio during playback.
   */
  private visualize(): void {
    if (!this.isPlaying || !this.analyserNode || !this.audioContext || !this.audioBuffer) return;

    this.animationId = requestAnimationFrame(() => this.visualize());

    const dataArray = new Uint8Array(this.analyserNode.frequencyBinCount);
    this.analyserNode.getByteFrequencyData(dataArray);

    const width = this.canvas.width;
    const height = this.canvas.height;

    this.ctx.clearRect(0, 0, width, height);
    this.ctx.fillStyle = "#1e1e1e";
    this.ctx.fillRect(0, 0, width, height);

    // Draw frequency bars
    const barWidth = (width / dataArray.length) * 2.5;
    let x = 0;

    for (let i = 0; i < dataArray.length; i++) {
      const barHeight = (dataArray[i] / 255) * height;
      this.ctx.fillStyle = `rgb(74, 158, 255)`;
      this.ctx.fillRect(x, height - barHeight, barWidth, barHeight);
      x += barWidth + 1;
    }

    // Update progress
    const elapsed = this.audioContext.currentTime - this.startTime;
    const progress = elapsed / this.audioBuffer.duration;
    this.updateProgress(Math.min(progress, 1));
    this.updateDuration(elapsed, this.audioBuffer.duration);
  }

  /**
   * Update the progress bar.
   */
  private updateProgress(progress: number): void {
    const fill = this.progressBar.querySelector("#progress-fill") as HTMLDivElement;
    if (fill) {
      fill.style.width = `${progress * 100}%`;
    }
  }

  /**
   * Update the duration display.
   */
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

  /**
   * Update sample rate info display.
   */
  private updateSampleInfo(sampleRate: number, channels: number): void {
    const display = this.container.querySelector("#duration-display");
    if (display) {
      const current = display.textContent?.split(" • ")[0] || "0:00 / 0:00";
      const channelStr = channels === 1 ? "mono" : channels === 2 ? "stereo" : `${channels}ch`;
      display.textContent = `${current} • ${sampleRate}Hz ${channelStr} • Preview`;
    }
  }

  /**
   * Handle container resize.
   */
  private handleResize(): void {
    this.canvas.width = this.container.clientWidth - 24;
    this.canvas.height = Math.max(100, this.container.clientHeight - 80);
    if (this.audioBuffer) {
      this.drawWaveform();
    } else {
      this.drawEmptyWaveform();
    }
  }

  /**
   * Clear the preview.
   */
  clear(): void {
    this.stop();
    this.audioBuffer = null;
    this.drawEmptyWaveform();
  }

  /**
   * Dispose of the preview and release resources.
   */
  dispose(): void {
    this.stop();
    if (this.audioContext) {
      this.audioContext.close();
    }
    this.container.innerHTML = "";
  }
}
