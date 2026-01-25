/**
 * SpecCade Editor main entry point.
 *
 * This module initializes the editor application:
 * - Sets up Monaco editor with Starlark syntax highlighting
 * - Connects to the Tauri backend for spec evaluation
 * - Manages preview rendering for different asset types
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Editor, EditorDiagnostic } from "./components/Editor";
import { MeshPreview } from "./components/MeshPreview";
import { AudioPreview } from "./components/AudioPreview";
import { TexturePreview } from "./components/TexturePreview";
import { NewAssetDialog } from "./components/NewAssetDialog";
import { FileBrowser } from "./components/FileBrowser";

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

// Types for file watching
interface FileChangeEvent {
  path: string;
  kind: "created" | "modified" | "removed";
}

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
let editor: Editor | null = null;
let meshPreview: MeshPreview | null = null;
let audioPreview: AudioPreview | null = null;
let texturePreview: TexturePreview | null = null;
let fileBrowser: FileBrowser | null = null;
let currentAssetType: string | null = null;
let currentWatchedPath: string | null = null;
let currentFilePath: string | null = null;
let fileChangeUnlisten: UnlistenFn | null = null;

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

// DOM elements
const editorContainer = document.getElementById("editor-container")!;
const statusBar = document.getElementById("status-bar")!;
const previewContent = document.getElementById("preview-content")!;

/**
 * Watch a file for external changes.
 */
export async function watchFile(path: string): Promise<void> {
  if (currentWatchedPath === path) return;

  try {
    await invoke("plugin:speccade|watch_file", { path });
    currentWatchedPath = path;
  } catch (error) {
    console.error("Failed to watch file:", error);
  }
}

/**
 * Stop watching the current file.
 */
export async function unwatchFile(): Promise<void> {
  if (!currentWatchedPath) return;

  try {
    await invoke("plugin:speccade|unwatch_file");
    currentWatchedPath = null;
  } catch (error) {
    console.error("Failed to unwatch file:", error);
  }
}

/**
 * Get the currently open file path.
 */
export function getCurrentFilePath(): string | null {
  return currentFilePath;
}

/**
 * Cleanup resources when the editor is disposed.
 */
export function cleanup(): void {
  if (fileChangeUnlisten) {
    fileChangeUnlisten();
    fileChangeUnlisten = null;
  }
  if (editor) {
    editor.dispose();
    editor = null;
  }
  if (fileBrowser) {
    fileBrowser.dispose();
    fileBrowser = null;
  }
  clearPreviewComponents();
}

/**
 * Parse location string to extract line/column.
 */
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

/**
 * Convert backend errors/warnings to editor diagnostics.
 */
function convertToDiagnostics(
  errors: EvalError[],
  warnings: EvalWarning[]
): EditorDiagnostic[] {
  const diagnostics: EditorDiagnostic[] = [];

  // Add error diagnostics
  for (const error of errors) {
    const loc = error.location ? parseLocation(error.location) : null;
    diagnostics.push({
      severity: "error",
      message: `${error.code}: ${error.message}`,
      startLine: loc?.line ?? 1,
      startColumn: loc?.column ?? 1,
    });
  }

  // Add warning diagnostics
  for (const warning of warnings) {
    const loc = warning.location ? parseLocation(warning.location) : null;
    diagnostics.push({
      severity: "warning",
      message: `${warning.code}: ${warning.message}`,
      startLine: loc?.line ?? 1,
      startColumn: loc?.column ?? 1,
    });
  }

  return diagnostics;
}

/**
 * Update status bar text.
 */
function updateStatus(message: string): void {
  statusBar.textContent = message;
}

/**
 * Update window title with current file path.
 */
function updateWindowTitle(path: string | null): void {
  if (path) {
    // Extract just the filename from the full path
    const filename = path.split(/[\\/]/).pop() ?? path;
    document.title = `${filename} - SpecCade Editor`;
  } else {
    document.title = "SpecCade Editor";
  }
}

/**
 * Clear all preview components.
 */
function clearPreviewComponents(): void {
  if (meshPreview) {
    meshPreview.dispose();
    meshPreview = null;
  }
  if (audioPreview) {
    audioPreview.dispose();
    audioPreview = null;
  }
  if (texturePreview) {
    texturePreview.dispose();
    texturePreview = null;
  }
  previewContent.innerHTML = "";
}

/**
 * Evaluate the current source and update diagnostics.
 */
async function evaluateSource(content: string): Promise<void> {
  if (!content.trim()) {
    updateStatus("Ready");
    if (editor) editor.clearDiagnostics();
    return;
  }

  updateStatus("Evaluating...");

  try {
    const result = await invoke<EvalOutput>("plugin:speccade|eval_spec", {
      source: content,
      filename: "editor.star",
    });

    if (result.success) {
      const hashPreview = result.source_hash?.slice(0, 8) ?? "";
      updateStatus(`OK - ${hashPreview}...`);
      await updatePreview(result.result, content);
      if (editor) {
        editor.setDiagnostics(convertToDiagnostics([], result.warnings));
      }
    } else {
      const errorCount = result.errors.length;
      updateStatus(`${errorCount} error${errorCount === 1 ? "" : "s"}`);
      await updatePreview(null, content);
      if (editor) {
        editor.setDiagnostics(convertToDiagnostics(result.errors, result.warnings));
      }
    }
  } catch (error) {
    updateStatus(`IPC Error: ${error}`);
    await updatePreview(null, content);
    if (editor) {
      editor.setDiagnostics([
        {
          severity: "error",
          message: `IPC_ERROR: ${String(error)}`,
          startLine: 1,
          startColumn: 1,
        },
      ]);
    }
  }
}

/**
 * Update preview pane based on evaluation result.
 */
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

/**
 * Render mesh preview with three.js.
 */
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

/**
 * Render audio preview with Web Audio.
 */
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

/**
 * Render texture preview as image.
 */
async function renderTexturePreview(source: string): Promise<void> {
  // Create texture preview if needed
  if (!texturePreview) {
    previewContent.innerHTML = "";
    texturePreview = new TexturePreview(previewContent);
  }

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
      await texturePreview.loadTexture(
        preview.data,
        preview.mime_type ?? "image/png",
        preview.metadata as Record<string, unknown> | undefined
      );
      updateStatus("Texture preview ready");
    } else {
      updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
    }
  } catch (error) {
    updateStatus(`Preview error: ${error}`);
  }
}

/**
 * Render JSON preview for unknown asset types.
 */
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

/**
 * Initialize the editor component.
 */
function initEditor(): void {
  // Create the Editor component with 500ms debounce for validation
  editor = new Editor(editorContainer, {
    initialContent: DEFAULT_SPEC,
    debounceMs: 500, // 500ms debounce as per acceptance criteria
    onChange: (event) => {
      // Evaluate source when content changes (after 500ms debounce)
      evaluateSource(event.content);
    },
  });

  // Trigger initial evaluation
  evaluateSource(editor.getContent());
}

/**
 * Initialize the application.
 */
async function init(): Promise<void> {
  // Listen for file change events from backend
  fileChangeUnlisten = await listen<FileChangeEvent>("file-changed", (event) => {
    const { kind } = event.payload;
    if (kind === "modified" && editor) {
      updateStatus(`External change detected`);
      evaluateSource(editor.getContent());
    }
  });

  initEditor();

  // Initialize file browser in sidebar
  const sidebarElement = document.getElementById("sidebar");
  if (sidebarElement) {
    fileBrowser = new FileBrowser(sidebarElement, (path, content) => {
      currentFilePath = path;
      if (editor) {
        editor.setContent(content);
        evaluateSource(content);
      }
      updateWindowTitle(path);
    });
  }

  // Setup New Asset button click handler
  const newAssetBtn = document.getElementById("new-asset-btn");
  if (newAssetBtn) {
    newAssetBtn.addEventListener("click", () => {
      const dialog = new NewAssetDialog((content) => {
        if (editor) {
          editor.setContent(content);
          evaluateSource(content);
        }
        // Clear current file path when creating new asset
        currentFilePath = null;
        updateWindowTitle(null);
      });
      dialog.show();
    });
  }

  updateStatus("Ready");
}

// Start the application when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
