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
import { save } from "@tauri-apps/plugin-dialog";
import { Editor, EditorDiagnostic } from "./components/Editor";
import { MeshPreview } from "./components/MeshPreview";
import { AudioPreview } from "./components/AudioPreview";
import { TexturePreview } from "./components/TexturePreview";
import { MusicPreview } from "./components/MusicPreview";
import { AnimationPreview, type AnimationMetadata } from "./components/AnimationPreview";
import { NewAssetDialog } from "./components/NewAssetDialog";
import { FileBrowser } from "./components/FileBrowser";
import { ProjectExplorer } from "./components/ProjectExplorer";
import { GeneratePanel } from "./components/GeneratePanel";
import { StdlibPalette } from "./components/StdlibPalette";
import { HelpPanel } from "./components/HelpPanel";
import { ProblemsPanel, ProblemItem, ProblemStage } from "./components/ProblemsPanel";
import { addRecentFile } from "./lib/recent-files";

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

/** A lint issue from the backend. */
interface LintIssue {
  rule_id: string;
  severity: "error" | "warning" | "info";
  message: string;
  suggestion: string;
  spec_path?: string;
  actual_value?: string;
  expected_range?: string;
}

/** Lint output from the backend. */
interface LintOutput {
  ok: boolean;
  issues: LintIssue[];
  error_count: number;
  warning_count: number;
  info_count: number;
}

interface PreviewResult {
  success: boolean;
  asset_type: string;
  data?: string; // base64 encoded
  mime_type?: string;
  error?: string;
  metadata?: Record<string, unknown>;
  quality?: "proxy" | "full";
  can_refine?: boolean;
  lint?: LintOutput;
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
let musicPreview: MusicPreview | null = null;
let texturePreview: TexturePreview | null = null;
let animationPreview: AnimationPreview | null = null;
let fileBrowser: FileBrowser | null = null;
let projectExplorer: ProjectExplorer | null = null;
let generatePanel: GeneratePanel | null = null;
let problemsPanel: ProblemsPanel | null = null;
let currentProblems: ProblemItem[] = [];
let stdlibPalette: StdlibPalette | null = null;
let currentAssetType: string | null = null;
let currentWatchedPath: string | null = null;
let currentFilePath: string | null = null;
let fileChangeUnlisten: UnlistenFn | null = null;
let rightPaneTabsAbort: AbortController | null = null;

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

  if (rightPaneTabsAbort) {
    rightPaneTabsAbort.abort();
    rightPaneTabsAbort = null;
  }

  if (editor) {
    editor.dispose();
    editor = null;
  }
  if (fileBrowser) {
    fileBrowser.dispose();
    fileBrowser = null;
  }
  if (projectExplorer) {
    projectExplorer.dispose();
    projectExplorer = null;
  }
  if (generatePanel) {
    generatePanel.dispose();
    generatePanel = null;
  }
  if (problemsPanel) {
    problemsPanel.dispose();
    problemsPanel = null;
  }
  if (stdlibPalette) {
    stdlibPalette.dispose();
    stdlibPalette = null;
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
 * Convert backend eval errors/warnings to Problems panel items.
 */
function convertToProblems(errors: EvalError[], warnings: EvalWarning[]): ProblemItem[] {
  const problems: ProblemItem[] = [];

  for (let i = 0; i < errors.length; i++) {
    const error = errors[i];
    const loc = error.location ? parseLocation(error.location) : null;
    problems.push({
      id: `eval:error:${error.code}:${i}`,
      stage: "eval",
      severity: "error",
      code: error.code,
      message: error.message,
      location: loc ?? undefined,
    });
  }

  for (let i = 0; i < warnings.length; i++) {
    const warning = warnings[i];
    const loc = warning.location ? parseLocation(warning.location) : null;
    problems.push({
      id: `eval:warning:${warning.code}:${i}`,
      stage: "eval",
      severity: "warning",
      code: warning.code,
      message: warning.message,
      location: loc ?? undefined,
    });
  }

  return problems;
}

function setProblems(items: ProblemItem[]): void {
  currentProblems = items.slice();
  problemsPanel?.setProblems(currentProblems);
}

function replaceStage(stage: ProblemStage, itemsForStage: ProblemItem[]): void {
  const next: ProblemItem[] = [];
  let inserted = false;

  for (const problem of currentProblems) {
    if (problem.stage === stage) {
      if (!inserted) {
        next.push(...itemsForStage);
        inserted = true;
      }
      continue;
    }
    next.push(problem);
  }

  if (!inserted) {
    next.push(...itemsForStage);
  }

  setProblems(next);
}

function compileProblem(message: string): ProblemItem {
  return {
    id: "compile:error:COMPILE_ERROR:0",
    stage: "compile",
    severity: "error",
    code: "COMPILE_ERROR",
    message,
  };
}

function ipcProblem(message: string): ProblemItem {
  return {
    id: "ipc:error:IPC_ERROR:0",
    stage: "ipc",
    severity: "error",
    code: "IPC_ERROR",
    message,
  };
}

function previewProblem(message: string): ProblemItem {
  return {
    id: "preview:error:PREVIEW_ERROR:0",
    stage: "preview",
    severity: "error",
    code: "PREVIEW_ERROR",
    message,
  };
}

/**
 * Convert lint output to Problems panel items.
 */
function convertLintToProblems(lint: LintOutput): ProblemItem[] {
  const problems: ProblemItem[] = [];

  for (let i = 0; i < lint.issues.length; i++) {
    const issue = lint.issues[i];
    // Map lint severity to problem severity (info becomes warning in the panel)
    const severity: "error" | "warning" =
      issue.severity === "error" ? "error" : "warning";

    // Build the message: include suggestion for actionable feedback
    let message = issue.message;
    if (issue.actual_value && issue.expected_range) {
      message += ` (got ${issue.actual_value}, expected ${issue.expected_range})`;
    }
    if (issue.suggestion) {
      message += `. ${issue.suggestion}`;
    }

    problems.push({
      id: `lint:${issue.severity}:${issue.rule_id}:${i}`,
      stage: "lint",
      severity,
      code: issue.rule_id,
      message,
      // Lint issues don't have source line/column, but may have spec_path
      location: undefined,
    });
  }

  return problems;
}

/**
 * Process lint results from a preview and update the Problems panel.
 */
function processLintResults(preview: PreviewResult | undefined): void {
  if (preview?.lint) {
    const lintProblems = convertLintToProblems(preview.lint);
    replaceStage("lint", lintProblems);
  } else {
    replaceStage("lint", []);
  }
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
 * Save the current file.
 * If a file path exists, saves to that path. Otherwise shows Save As dialog.
 */
async function saveCurrentFile(): Promise<void> {
  if (!editor) return;

  const content = editor.getContent();

  if (currentFilePath) {
    // Save to existing file
    try {
      await invoke("plugin:speccade|save_file", { path: currentFilePath, content });
      updateStatus("Saved");
    } catch (error) {
      updateStatus(`Save error: ${error}`);
    }
  } else {
    // Show Save As dialog
    try {
      const filePath = await save({
        filters: [{ name: "Starlark Files", extensions: ["star"] }],
      });

      if (filePath) {
        await invoke("plugin:speccade|save_file", { path: filePath, content });
        currentFilePath = filePath;
        updateWindowTitle(filePath);
        updateStatus("Saved");
      }
    } catch (error) {
      updateStatus(`Save error: ${error}`);
    }
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
  if (musicPreview) {
    musicPreview.dispose();
    musicPreview = null;
  }
  if (texturePreview) {
    texturePreview.dispose();
    texturePreview = null;
  }
  if (animationPreview) {
    animationPreview.dispose();
    animationPreview = null;
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
    setProblems([]);
    return;
  }

  updateStatus("Evaluating...");

  try {
    const filename = currentFilePath ?? "editor.star";
    const result = await invoke<EvalOutput>("plugin:speccade|eval_spec", {
      source: content,
      filename,
    });

    if (result.success) {
      const hashPreview = result.source_hash?.slice(0, 8) ?? "";
      updateStatus(`OK - ${hashPreview}...`);
      if (editor) {
        editor.setDiagnostics(convertToDiagnostics([], result.warnings));
      }

      replaceStage("ipc", []);
      replaceStage("eval", convertToProblems([], result.warnings));
      await updatePreview(result.result, content);
    } else {
      const errorCount = result.errors.length;
      updateStatus(`${errorCount} error${errorCount === 1 ? "" : "s"}`);
      if (editor) {
        editor.setDiagnostics(convertToDiagnostics(result.errors, result.warnings));
      }

      replaceStage("ipc", []);
      replaceStage("compile", []);
      replaceStage("preview", []);
      replaceStage("eval", convertToProblems(result.errors, result.warnings));
      await updatePreview(null, content);
    }
  } catch (error) {
    updateStatus(`IPC Error: ${error}`);
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

    replaceStage("ipc", [ipcProblem(String(error))]);
  }
}

/**
 * Update preview pane based on evaluation result.
 */
async function updatePreview(result: unknown, source: string): Promise<void> {
  if (result === null || result === undefined) {
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
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
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
    currentAssetType = assetType ?? null;
  }

  // Handle different asset types
  if (assetType === "audio") {
    await renderAudioPreview(source);
  } else if (assetType === "music") {
    await renderMusicPreview(source);
  } else if (
    assetType === "texture" ||
    assetType === "sprite" ||
    assetType === "ui" ||
    assetType === "font" ||
    assetType === "vfx"
  ) {
    await renderTexturePreview(source);
  } else if (
    assetType === "static_mesh" ||
    assetType === "skeletal_mesh" ||
    assetType === "mesh"
  ) {
    await renderMeshPreview(source);
  } else if (assetType === "skeletal_animation") {
    await renderAnimationPreview(source);
  } else {
    // Fallback to JSON preview for unknown types
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
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

  const filename = currentFilePath ?? "editor.star";
  updateStatus("Generating mesh preview...");

  let result: GeneratePreviewOutput;
  try {
    result = await invoke<GeneratePreviewOutput>("plugin:speccade|generate_preview", {
      source,
      filename,
    });
  } catch (error) {
    replaceStage("ipc", [ipcProblem(String(error))]);
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`IPC Error: ${error}`);
    return;
  }

  if (!result.compile_success) {
    replaceStage("compile", [compileProblem(result.compile_error ?? "Unknown error")]);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
    return;
  }

  replaceStage("compile", []);

  const preview = result.preview;
  if (preview?.success && preview.data) {
    replaceStage("preview", []);
    processLintResults(preview);
    try {
      await meshPreview.loadGLB(preview.data);
      updateStatus("Mesh preview ready");
    } catch (error) {
      replaceStage("preview", [previewProblem(String(error))]);
      updateStatus(`Preview error: ${error}`);
    }
  } else {
    replaceStage("preview", [previewProblem(preview?.error ?? "Unknown error")]);
    replaceStage("lint", []);
    updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
  }
}

/**
 * Render animation preview with three.js.
 */
async function renderAnimationPreview(source: string): Promise<void> {
  // Create animation preview if needed
  if (!animationPreview) {
    previewContent.innerHTML = "";
    animationPreview = new AnimationPreview(previewContent);
  }

  const filename = currentFilePath ?? "editor.star";
  updateStatus("Generating animation preview...");
  animationPreview.showGenerating();

  let result: GeneratePreviewOutput;
  try {
    result = await invoke<GeneratePreviewOutput>("plugin:speccade|generate_preview", {
      source,
      filename,
    });
  } catch (error) {
    replaceStage("ipc", [ipcProblem(String(error))]);
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`IPC Error: ${error}`);
    return;
  }

  if (!result.compile_success) {
    replaceStage("compile", [compileProblem(result.compile_error ?? "Unknown error")]);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
    return;
  }

  replaceStage("compile", []);

  const preview = result.preview;
  if (preview?.success && preview.data) {
    replaceStage("preview", []);
    processLintResults(preview);
    try {
      const metadata = preview.metadata as AnimationMetadata | undefined;
      await animationPreview.loadGLB(preview.data, metadata, filename);
      updateStatus("Animation preview ready");
    } catch (error) {
      replaceStage("preview", [previewProblem(String(error))]);
      updateStatus(`Preview error: ${error}`);
    }
  } else {
    replaceStage("preview", [previewProblem(preview?.error ?? "Unknown error")]);
    replaceStage("lint", []);
    updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
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

  const filename = currentFilePath ?? "editor.star";
  updateStatus("Generating audio preview...");

  let result: GeneratePreviewOutput;
  try {
    result = await invoke<GeneratePreviewOutput>("plugin:speccade|generate_preview", {
      source,
      filename,
    });
  } catch (error) {
    replaceStage("ipc", [ipcProblem(String(error))]);
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`IPC Error: ${error}`);
    return;
  }

  if (!result.compile_success) {
    replaceStage("compile", [compileProblem(result.compile_error ?? "Unknown error")]);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
    return;
  }

  replaceStage("compile", []);

  const preview = result.preview;
  if (preview?.success && preview.data) {
    replaceStage("preview", []);
    processLintResults(preview);
    try {
      await audioPreview.loadWAV(preview.data, filename);
      updateStatus("Audio preview ready");
    } catch (error) {
      replaceStage("preview", [previewProblem(String(error))]);
      updateStatus(`Preview error: ${error}`);
    }
  } else {
    replaceStage("preview", [previewProblem(preview?.error ?? "Unknown error")]);
    replaceStage("lint", []);
    updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
  }
}

/**
 * Render music preview with chiptune3.
 */
async function renderMusicPreview(source: string): Promise<void> {
  const filename = currentFilePath ?? "editor.star";

  if (!musicPreview) {
    previewContent.innerHTML = "";
    musicPreview = new MusicPreview(previewContent, async (src, reqFilename) => {
      let didInvoke = false;
      try {
        const result = await invoke<GeneratePreviewOutput>(
          "plugin:speccade|generate_preview",
          { source: src, filename: reqFilename }
        );
        didInvoke = true;

        // IPC succeeded; clear any stale IPC errors.
        replaceStage("ipc", []);

        if (!result.compile_success) {
          replaceStage("compile", [
            compileProblem(result.compile_error ?? "Unknown compile error"),
          ]);
          replaceStage("preview", []);
          throw new Error(result.compile_error ?? "Unknown compile error");
        }

        replaceStage("compile", []);

        const preview = result.preview;
        if (!preview?.success || !preview.data) {
          replaceStage("preview", [previewProblem(preview?.error ?? "Unknown preview error")]);
          replaceStage("lint", []);
          throw new Error(preview?.error ?? "Unknown preview error");
        }

        replaceStage("preview", []);
        replaceStage("ipc", []);
        processLintResults(preview);

        return {
          dataBase64: preview.data,
          mimeType: preview.mime_type,
          metadata: preview.metadata ?? undefined,
        };
      } catch (error) {
        if (!didInvoke) {
          replaceStage("ipc", [ipcProblem(String(error))]);
          replaceStage("compile", []);
          replaceStage("preview", []);
          replaceStage("lint", []);
        }
        throw error;
      }
    });
  }

  musicPreview.setSource(source, filename);
  musicPreview.onSourceUpdated();
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

  const filename = currentFilePath ?? "editor.star";
  updateStatus("Generating texture preview...");

  let result: GeneratePreviewOutput;
  try {
    result = await invoke<GeneratePreviewOutput>("plugin:speccade|generate_preview", {
      source,
      filename,
    });
  } catch (error) {
    replaceStage("ipc", [ipcProblem(String(error))]);
    replaceStage("compile", []);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`IPC Error: ${error}`);
    return;
  }

  if (!result.compile_success) {
    replaceStage("compile", [compileProblem(result.compile_error ?? "Unknown error")]);
    replaceStage("preview", []);
    replaceStage("lint", []);
    updateStatus(`Compile error: ${result.compile_error ?? "Unknown error"}`);
    return;
  }

  replaceStage("compile", []);

  const preview = result.preview;
  if (preview?.success && preview.data) {
    replaceStage("preview", []);
    processLintResults(preview);
    try {
      await texturePreview.loadTexture(
        preview.data,
        preview.mime_type ?? "image/png",
        preview.metadata as Record<string, unknown> | undefined,
        {
          filePath: filename,
          assetType: currentAssetType ?? "texture",
          quality: preview.quality ?? "proxy",
        }
      );
      updateStatus("Texture preview ready");
    } catch (error) {
      replaceStage("preview", [previewProblem(String(error))]);
      updateStatus(`Preview error: ${error}`);
    }
  } else {
    replaceStage("preview", [previewProblem(preview?.error ?? "Unknown error")]);
    replaceStage("lint", []);
    updateStatus(`Preview error: ${preview?.error ?? "Unknown error"}`);
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
    // Refresh project tree on any file system change
    if (kind === "created" || kind === "removed") {
      projectExplorer?.refresh();
    }
  });

  // Initialize problems panel before the first evaluation runs.
  const problemsContentEl = document.getElementById("problems-content");
  if (problemsContentEl) {
    problemsPanel = new ProblemsPanel(problemsContentEl, (problem) => {
      if (!problem.location) return;
      editor?.selectAt(problem.location.line, problem.location.column);
      editor?.focus();
    });
  }

  initEditor();

  // Initialize project explorer in files panel (replaces FileBrowser)
  const filesPanelElement = document.getElementById("files-panel");
  if (filesPanelElement) {
    projectExplorer = new ProjectExplorer(filesPanelElement, (path, content) => {
      currentFilePath = path;
      addRecentFile(path);
      if (editor) {
        editor.setContent(content);
        evaluateSource(content);
      }
      updateWindowTitle(path);
      // setActiveFile is called internally by ProjectExplorer with the relative path
    });
  }

  // Initialize stdlib palette in snippets panel
  const snippetsPanelElement = document.getElementById("snippets-panel");
  if (snippetsPanelElement) {
    stdlibPalette = new StdlibPalette(snippetsPanelElement, (snippet: string) => {
      if (editor) {
        const monacoEditor = editor.getMonacoEditor();
        const selection = monacoEditor.getSelection();
        if (selection) {
          monacoEditor.executeEdits("stdlib-palette", [
            {
              range: selection,
              text: snippet,
              forceMoveMarkers: true,
            },
          ]);
        }
        editor.focus();
      }
    });
  }

  // Setup sidebar tab switching
  const filesTab = document.getElementById("files-tab");
  const snippetsTab = document.getElementById("snippets-tab");
  const filesPanel = document.getElementById("files-panel");
  const snippetsPanel = document.getElementById("snippets-panel");

  if (filesTab && snippetsTab && filesPanel && snippetsPanel) {
    filesTab.addEventListener("click", () => {
      // Show files panel, hide snippets panel
      filesPanel.style.display = "block";
      snippetsPanel.style.display = "none";
      // Update tab styles
      filesTab.style.background = "#252525";
      filesTab.style.color = "#fff";
      snippetsTab.style.background = "#1a1a1a";
      snippetsTab.style.color = "#888";
    });

    snippetsTab.addEventListener("click", () => {
      // Show snippets panel, hide files panel
      filesPanel.style.display = "none";
      snippetsPanel.style.display = "block";
      // Update tab styles
      snippetsTab.style.background = "#252525";
      snippetsTab.style.color = "#fff";
      filesTab.style.background = "#1a1a1a";
      filesTab.style.color = "#888";
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

  // Setup Save button click handler
  const saveBtn = document.getElementById("save-btn");
  if (saveBtn) {
    saveBtn.addEventListener("click", () => {
      saveCurrentFile();
    });
  }

  // Setup Help button click handler
  const helpBtn = document.getElementById("help-btn");
  if (helpBtn) {
    helpBtn.addEventListener("click", () => {
      const helpPanel = new HelpPanel();
      helpPanel.show();
    });
  }

  // Initialize generate panel
  const generateContent = document.getElementById("generate-content");
  if (generateContent) {
    generatePanel = new GeneratePanel(
      generateContent,
      () => editor?.getContent() ?? "",
      () => currentFilePath ?? "editor.star",
      (lint) => {
        if (lint) {
          const lintProblems = convertLintToProblems(lint);
          replaceStage("lint", lintProblems);
        } else {
          replaceStage("lint", []);
        }
      }
    );
  }

  // Setup tab switching
  const previewTab = document.getElementById("preview-tab");
  const generateTab = document.getElementById("generate-tab");
  const problemsTab = document.getElementById("problems-tab");
  const previewContentEl = document.getElementById("preview-content");
  const generateContentEl = document.getElementById("generate-content");

  type RightPaneTab = "preview" | "generate" | "problems";

  function setTabButtonActive(el: HTMLElement, active: boolean): void {
    el.style.background = active ? "#252525" : "#1a1a1a";
    el.style.color = active ? "#fff" : "#888";
  }

  function setRightPaneTab(tab: RightPaneTab): void {
    if (!previewContentEl || !generateContentEl || !problemsContentEl) return;

    previewContentEl.style.display = tab === "preview" ? "flex" : "none";
    generateContentEl.style.display = tab === "generate" ? "flex" : "none";
    problemsContentEl.style.display = tab === "problems" ? "flex" : "none";

    if (previewTab) setTabButtonActive(previewTab, tab === "preview");
    if (generateTab) setTabButtonActive(generateTab, tab === "generate");
    if (problemsTab) setTabButtonActive(problemsTab, tab === "problems");
  }

  if (previewTab && generateTab && problemsTab && previewContentEl && generateContentEl && problemsContentEl) {
    rightPaneTabsAbort?.abort();
    rightPaneTabsAbort = new AbortController();
    const { signal } = rightPaneTabsAbort;

    previewTab.addEventListener("click", () => setRightPaneTab("preview"), { signal });
    generateTab.addEventListener("click", () => setRightPaneTab("generate"), { signal });
    problemsTab.addEventListener("click", () => setRightPaneTab("problems"), { signal });
    setRightPaneTab("preview");
  }

  // Keyboard shortcuts
  document.addEventListener("keydown", (e) => {
    // Ctrl/Cmd+N for new asset
    if ((e.ctrlKey || e.metaKey) && e.key === "n") {
      e.preventDefault();
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
    }

    // Ctrl/Cmd+S for save
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      saveCurrentFile();
    }

    // F1 for help panel
    if (e.key === "F1") {
      e.preventDefault();
      const helpPanel = new HelpPanel();
      helpPanel.show();
    }
  });

  updateStatus("Ready");
}

// Start the application when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
