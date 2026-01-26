declare module "chiptune3" {
  export interface ChiptuneJsPlayerConfig {
    repeatCount?: number;
    stereoSeparation?: number;
    interpolationFilter?: number;
    context?: AudioContext;
  }

  export type ChiptuneProgressEvent = {
    pos?: number;
    order?: number;
    pattern?: number;
    row?: number;
  };

  // Minimal typing for our usage.
  export class ChiptuneJsPlayer {
    constructor(cfg?: ChiptuneJsPlayerConfig);

    onInitialized(handler: () => void): void;
    onEnded(handler: () => void): void;
    onError(handler: (e: unknown) => void): void;
    onMetadata(handler: (meta: Record<string, unknown> & { dur?: number }) => void): void;
    onProgress(handler: (e: ChiptuneProgressEvent) => void): void;

    play(data: ArrayBuffer): void;
    stop(): void;
    pause(): void;
    unpause(): void;
    togglePause(): void;
    setPos(seconds: number): void;
    seek(seconds: number): void;
    setVol(vol: number): void;
    getCurrentTime(): number | undefined;

    // Extended controls (present in chiptune3@0.8.x)
    setRepeatCount(val: number): void;
    setOrderRow(order: number, row: number): void;

    // Implementation detail in chiptune3; used for cleanup.
    context: AudioContext;
    duration?: number;
  }
}
