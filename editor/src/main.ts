import * as monaco from "monaco-editor";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface FileChangeEvent {
  path: string;
  kind: string;
}

// Current watched file path
let currentWatchedPath: string | null = null;
import {
  registerStarlarkLanguage,
  STARLARK_LANGUAGE_ID,
} from "./lib/starlark";
import { MeshPreview } from "./components/MeshPreview";
import { AudioPreview } from "./components/AudioPreview";

// Configure Monaco worker paths for Vite
self.MonacoEnvironment = {
  getWorker(_workerId: string, _label: string) {
    // Return a simple worker - language features are handled by our Rust backend
    return new Worker(
      new URL("monaco-editor/esm/vs/editor/editor.worker.js", import.meta.url),
      { type: "module" }
    );
  },
};

// Types for IPC communication
interface EvalError {
  code: string;
  message: string;
  location?: string;
}

interface EvalWarning {
  code: string;
  message: string;
  location?: string;
}

interface EvalOutput {
  success: boolean;
  errors: EvalError[];
  warnings: EvalWarning[];
  result?: unknown;
  source_hash?: string;
}

interface PreviewResult {
  success: boolean;
  asset_type: string;
  data?: string; // base64 encoded
  mime_type?: string;
  error?: string;
  metadata?: Record<string, unknown>;
}

interface GeneratePreviewOutput {
  compile_success: boolean;
  compile_error?: string;
  preview?: PreviewResult;
}

// Spec result type for asset_type detection
interface SpecResult {
  asset_type?: string;
  [key: string]: unknown;
}

// Application state
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let evalTimeout: number | null = null;
let meshPreview: MeshPreview | null = null;
let audioPreview: AudioPreview | null = null;
let currentAssetType: string | null = null;

// Watch a file for external changes
export async function watchFile(path: string): Promise<void> {
  if (currentWatchedPath === path) return;

  try {
    await invoke("plugin:speccade|watch_file", { path });
    currentWatchedPath = path;
  } catch (error) {
    console.error("Failed to watch file:", error);
  }
}

// Stop watching the current file
export async function unwatchFile(): Promise<void> {
  if (!currentWatchedPath) return;

  try {
    await invoke("plugin:speccade|unwatch_file");
    currentWatchedPath = null;
  } catch (error) {
    console.error("Failed to unwatch file:", error);
  }
}

// DOM elements
const editorContainer = document.getElementById("editor-container")!;
const statusBar = document.getElementById("status-bar")!;
const previewContent = document.getElementById("preview-content")!;

// Default spec template
const DEFAULT_SPEC = `# SpecCade Starlark Spec
# Define your asset using the SpecCade stdlib

{
    "spec_version": 1,
    "asset_id": "my-asset",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {"kind": "primary", "format": "wav", "path": "sounds/my-asset.wav"}
    ]
}
`;

// Initialize Monaco editor
function initEditor(): void {
  // Register Starlark language
  registerStarlarkLanguage();

  // Create the editor
  editor = monaco.editor.create(editorContainer, {
    value: DEFAULT_SPEC,
    language: STARLARK_LANGUAGE_ID,
    theme: "vs-dark",
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    lineNumbers: "on",
    renderWhitespace: "selection",
    scrollBeyondLastLine: false,
    wordWrap: "on",
    tabSize: 4,
    insertSpaces: true,
    padding: { top: 12, bottom: 12 },
  });

  // Listen for content changes
  editor.onDidChangeModelContent(() => {
    handleEditorChange();
  });

  // Initial evaluation
  evaluateSource();
}

// Handle editor content changes with debouncing
function handleEditorChange(): void {
  // Clear previous timeout
  if (evalTimeout !== null) {
    clearTimeout(evalTimeout);
  }

  // Debounce evaluation (50ms as per plan)
  evalTimeout = setTimeout(() => {
    evaluateSource();
  }, 50) as unknown as number;
}

// Parse location string to extract line/column
function parseLocation(location: string): { line: number; column: number } | null {
  // Try to parse "file.star:line:column" format
  const match = location.match(/:(\d+):(\d+)/);
  if (match) {
    return { line: parseInt(match[1], 10), column: parseInt(match[2], 10) };
  }
  // Try "line N" format
  const lineMatch = location.match(/line\s+(\d+)/i);
  if (lineMatch) {
    return { line: parseInt(lineMatch[1], 10), column: 1 };
  }
  return null;
}

// Update editor diagnostics (error markers)
function updateDiagnostics(errors: EvalError[], warnings: EvalWarning[]): void {
  if (!editor) return;

  const model = editor.getModel();
  if (!model) return;

  const markers: monaco.editor.IMarkerData[] = [];

  // Add error markers
  for (const error of errors) {
    const loc = error.location ? parseLocation(error.location) : null;
    const line = loc?.line ?? 1;
    const column = loc?.column ?? 1;

    markers.push({
      severity: monaco.MarkerSeverity.Error,
      message: `${error.code}: ${error.message}`,
      startLineNumber: line,
      startColumn: column,
      endLineNumber: line,
      endColumn: column + 10, // Approximate end
    });
  }

  // Add warning markers
  for (const warning of warnings) {
    const loc = warning.location ? parseLocation(warning.location) : null;
    const line = loc?.line ?? 1;
    const column = loc?.column ?? 1;

    markers.push({
      severity: monaco.MarkerSeverity.Warning,
      message: `${warning.code}: ${warning.message}`,
      startLineNumber: line,
      startColumn: column,
      endLineNumber: line,
      endColumn: column + 10,
    });
  }

  monaco.editor.setModelMarkers(model, "speccade", markers);
}

// Evaluate the current source
async function evaluateSource(): Promise<void> {
  if (!editor) return;

  const source = editor.getValue();
  if (!source.trim()) {
    updateStatus("Ready");
    updateDiagnostics([], []);
    return;
  }

  updateStatus("Evaluating...");

  try {
    const result = await invoke<EvalOutput>("plugin:speccade|eval_spec", {
      source,
      filename: "editor.star",
    });

    if (result.success) {
      const hashPreview = result.source_hash?.slice(0, 8) ?? "";
      updateStatus(`OK - ${hashPreview}...`);
      await updatePreview(result.result, source);
      updateDiagnostics([], result.warnings);
    } else {
      const errorCount = result.errors.length;
      updateStatus(`${errorCount} error${errorCount === 1 ? "" : "s"}`);
      await updatePreview(null, source);
      updateDiagnostics(result.errors, result.warnings);
    }
  } catch (error) {
    updateStatus(`IPC Error: ${error}`);
    await updatePreview(null, source);
    updateDiagnostics(
      [{ code: "IPC_ERROR", message: String(error) }],
      []
    );
  }
}

// Update status bar
function updateStatus(message: string): void {
  statusBar.textContent = message;
}

// Update preview pane
async function updatePreview(result: unknown, source: string): Promise<void> {
  if (result === null || result === undefined) {
    clearPreviewComponents();
    previewContent.innerHTML = `<span style="color: #666;">No preview available</span>`;
    return;
  }

  // Detect asset type from result
  const specResult = result as SpecResult;
  const assetType = specResult.asset_type;

  // If asset type changed, clear old preview components
  if (assetType !== currentAssetType) {
    clearPreviewComponents();
    currentAssetType = assetType ?? null;
  }

  // Handle different asset types
  if (assetType === "mesh") {
    await renderMeshPreview(source);
  } else if (assetType === "audio") {
    await renderAudioPreview(source);
  } else if (assetType === "texture") {
    await renderTexturePreview(source);
  } else {
    // Fallback to JSON preview for unknown types
    renderJsonPreview(result);
  }
}

// Clear preview components
function clearPreviewComponents(): void {
  if (meshPreview) {
    meshPreview.dispose();
    meshPreview = null;
  }
  if (audioPreview) {
    audioPreview.dispose();
    audioPreview = null;
  }
  previewContent.innerHTML = "";
}

// Render mesh preview with three.js
async function renderMeshPreview(source: string): Promise<void> {
  // Create mesh preview if needed
  if (!meshPreview) {
    previewContent.innerHTML = "";
    meshPreview = new MeshPreview(previewContent);
  }

  try {
    updateStatus("Generating mesh preview...");
    const result = await invoke<GeneratePreviewOutput>(
      "plugin:speccade|generate_preview",
      { source, filename: "editor.star" }
    );

    if (!result.compile_success) {
      updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
      return;
    }

    const preview = result.preview;
    if (preview?.success && preview.data) {
      await meshPreview.loadGLB(preview.data);
      updateStatus("Mesh preview ready");
    } else {
      updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
    }
  } catch (error) {
    updateStatus(`Preview error: ${error}`);
  }
}

// Render audio preview with Web Audio
async function renderAudioPreview(source: string): Promise<void> {
  // Create audio preview if needed
  if (!audioPreview) {
    previewContent.innerHTML = "";
    audioPreview = new AudioPreview(previewContent);
  }

  try {
    updateStatus("Generating audio preview...");
    const result = await invoke<GeneratePreviewOutput>(
      "plugin:speccade|generate_preview",
      { source, filename: "editor.star" }
    );

    if (!result.compile_success) {
      updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
      return;
    }

    const preview = result.preview;
    if (preview?.success && preview.data) {
      await audioPreview.loadWAV(preview.data);
      updateStatus("Audio preview ready");
    } else {
      updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
    }
  } catch (error) {
    updateStatus(`Preview error: ${error}`);
  }
}

// Render texture preview as image
async function renderTexturePreview(source: string): Promise<void> {
  clearPreviewComponents();

  try {
    updateStatus("Generating texture preview...");
    const result = await invoke<GeneratePreviewOutput>(
      "plugin:speccade|generate_preview",
      { source, filename: "editor.star" }
    );

    if (!result.compile_success) {
      updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
      return;
    }

    const preview = result.preview;
    if (preview?.success && preview.data) {
      const img = document.createElement("img");
      img.src = `data:${preview.mime_type ?? "image/png"};base64,${preview.data}`;
      img.style.cssText = `
        max-width: 100%;
        max-height: 100%;
        object-fit: contain;
        display: block;
        margin: auto;
      `;
      previewContent.innerHTML = "";
      previewContent.appendChild(img);
      updateStatus("Texture preview ready");
    } else {
      updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
    }
  } catch (error) {
    updateStatus(`Preview error: ${error}`);
  }
}

// Render JSON preview for unknown asset types
function renderJsonPreview(result: unknown): void {
  clearPreviewComponents();

  const pre = document.createElement("pre");
  pre.style.cssText = `
    margin: 0;
    padding: 12px;
    font-size: 11px;
    font-family: 'Consolas', 'Monaco', monospace;
    color: #9cdcfe;
    overflow: auto;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    white-space: pre-wrap;
    word-break: break-word;
  `;
  pre.textContent = JSON.stringify(result, null, 2);

  previewContent.innerHTML = "";
  previewContent.appendChild(pre);
}

// Initialize the application
function init(): void {
  // Listen for file change events from backend
  listen<FileChangeEvent>("file-changed", (event) => {
    const { path, kind } = event.payload;
    if (kind === "modified" && editor) {
      // Reload file content - for now just trigger re-evaluation
      // In future: prompt user or auto-reload
      updateStatus(`External change detected: ${path}`);
      evaluateSource();
    }
  });

  initEditor();
  updateStatus("Ready");
}

// Start the application when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
