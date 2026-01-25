/**
 * Monaco Editor wrapper component for Starlark spec editing.
 *
 * This component provides:
 * - Monaco editor instance with Starlark syntax highlighting
 * - Debounced content change events (500ms delay for validation)
 * - Inline error markers (red squiggles) from parse/validation errors
 * - File loading and saving capabilities
 */
import * as monaco from "monaco-editor";
import {
  STARLARK_LANGUAGE_ID,
  starlarkLanguageConfiguration,
} from "../lib/starlark-language";
import { starlarkMonarchTokens } from "../lib/starlark-tokens";
import { registerStarlarkCompletions } from "../lib/starlark-completions";

/**
 * Diagnostic information for editor markers.
 */
export interface EditorDiagnostic {
  /** Severity level (error, warning, info) */
  severity: "error" | "warning" | "info";
  /** Human-readable message */
  message: string;
  /** Start line (1-indexed) */
  startLine: number;
  /** Start column (1-indexed) */
  startColumn: number;
  /** End line (1-indexed, optional - defaults to startLine) */
  endLine?: number;
  /** End column (optional - defaults to startColumn + 10) */
  endColumn?: number;
}

/**
 * Editor change event payload.
 */
export interface EditorChangeEvent {
  /** The current editor content */
  content: string;
  /** Whether the content was changed by user typing (vs programmatic) */
  isUserInput: boolean;
}

/**
 * Configuration options for the Editor component.
 */
export interface EditorOptions {
  /** Initial content to display */
  initialContent?: string;
  /** Debounce delay in milliseconds for change events (default: 500ms) */
  debounceMs?: number;
  /** Callback when content changes (debounced) */
  onChange?: (event: EditorChangeEvent) => void;
  /** Whether the editor is read-only */
  readOnly?: boolean;
  /** Font size in pixels (default: 14) */
  fontSize?: number;
}

// Language registration flag to prevent duplicate registration
let starlarkRegistered = false;

/**
 * Register the Starlark language with Monaco (idempotent).
 */
function registerStarlarkLanguage(): void {
  if (starlarkRegistered) return;

  // Register the language
  monaco.languages.register({
    id: STARLARK_LANGUAGE_ID,
    extensions: [".star", ".bzl", ".bazel"],
    aliases: ["Starlark", "starlark", "star"],
    mimetypes: ["text/x-starlark"],
  });

  // Set language configuration (brackets, comments, etc.)
  monaco.languages.setLanguageConfiguration(
    STARLARK_LANGUAGE_ID,
    starlarkLanguageConfiguration
  );

  // Set token provider (syntax highlighting)
  monaco.languages.setMonarchTokensProvider(
    STARLARK_LANGUAGE_ID,
    starlarkMonarchTokens
  );

  // Register stdlib completions for autocomplete
  registerStarlarkCompletions();

  starlarkRegistered = true;
}

/**
 * Monaco editor wrapper for Starlark spec files.
 *
 * Provides syntax highlighting, inline error display, and debounced
 * change notifications for integration with the SpecCade validation backend.
 *
 * @example
 * ```typescript
 * const editor = new Editor(container, {
 *   initialContent: "# My spec\nspec(...)",
 *   debounceMs: 500,
 *   onChange: (event) => {
 *     console.log("Content changed:", event.content);
 *     validateWithBackend(event.content);
 *   },
 * });
 *
 * // Set errors from backend
 * editor.setDiagnostics([
 *   { severity: "error", message: "Invalid syntax", startLine: 2, startColumn: 1 },
 * ]);
 *
 * // Cleanup when done
 * editor.dispose();
 * ```
 */
export class Editor {
  private editor: monaco.editor.IStandaloneCodeEditor;
  private debounceTimeout: number | null = null;
  private debounceMs: number;
  private onChange?: (event: EditorChangeEvent) => void;
  private isSettingContent = false;

  /**
   * Create a new Editor instance.
   *
   * @param container - The DOM element to mount the editor in
   * @param options - Configuration options
   */
  constructor(container: HTMLElement, options: EditorOptions = {}) {
    this.debounceMs = options.debounceMs ?? 500;
    this.onChange = options.onChange;

    // Ensure Starlark language is registered
    registerStarlarkLanguage();

    // Create the Monaco editor instance
    this.editor = monaco.editor.create(container, {
      value: options.initialContent ?? "",
      language: STARLARK_LANGUAGE_ID,
      theme: "vs-dark",
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: options.fontSize ?? 14,
      lineNumbers: "on",
      renderWhitespace: "selection",
      scrollBeyondLastLine: false,
      wordWrap: "on",
      tabSize: 4,
      insertSpaces: true,
      padding: { top: 12, bottom: 12 },
      readOnly: options.readOnly ?? false,
      smoothScrolling: true,
      cursorBlinking: "smooth",
      cursorSmoothCaretAnimation: "on",
      renderLineHighlight: "line",
      bracketPairColorization: { enabled: true },
      guides: {
        bracketPairs: true,
        indentation: true,
      },
    });

    // Listen for content changes
    this.editor.onDidChangeModelContent(() => {
      this.handleContentChange();
    });
  }

  /**
   * Handle content changes with debouncing.
   *
   * Uses a 500ms debounce by default to prevent excessive validation
   * calls during rapid typing while still providing responsive feedback.
   */
  private handleContentChange(): void {
    // Skip if this was a programmatic content change
    if (this.isSettingContent) return;

    // Clear any pending timeout
    if (this.debounceTimeout !== null) {
      clearTimeout(this.debounceTimeout);
    }

    // Schedule debounced callback
    this.debounceTimeout = setTimeout(() => {
      this.debounceTimeout = null;
      if (this.onChange) {
        this.onChange({
          content: this.getContent(),
          isUserInput: true,
        });
      }
    }, this.debounceMs) as unknown as number;
  }

  /**
   * Get the current editor content.
   */
  getContent(): string {
    return this.editor.getValue();
  }

  /**
   * Set the editor content programmatically.
   *
   * This will NOT trigger the onChange callback.
   *
   * @param content - The new content to set
   */
  setContent(content: string): void {
    this.isSettingContent = true;
    this.editor.setValue(content);
    this.isSettingContent = false;
  }

  /**
   * Set diagnostics (errors/warnings) to display as inline markers.
   *
   * Clears any existing markers before setting new ones.
   *
   * @param diagnostics - Array of diagnostics to display
   */
  setDiagnostics(diagnostics: EditorDiagnostic[]): void {
    const model = this.editor.getModel();
    if (!model) return;

    const markers: monaco.editor.IMarkerData[] = diagnostics.map((d) => ({
      severity:
        d.severity === "error"
          ? monaco.MarkerSeverity.Error
          : d.severity === "warning"
            ? monaco.MarkerSeverity.Warning
            : monaco.MarkerSeverity.Info,
      message: d.message,
      startLineNumber: d.startLine,
      startColumn: d.startColumn,
      endLineNumber: d.endLine ?? d.startLine,
      endColumn: d.endColumn ?? d.startColumn + 10,
    }));

    monaco.editor.setModelMarkers(model, "speccade", markers);
  }

  /**
   * Clear all diagnostics from the editor.
   */
  clearDiagnostics(): void {
    const model = this.editor.getModel();
    if (!model) return;
    monaco.editor.setModelMarkers(model, "speccade", []);
  }

  /**
   * Focus the editor.
   */
  focus(): void {
    this.editor.focus();
  }

  /**
   * Get the underlying Monaco editor instance.
   *
   * Use sparingly - prefer the wrapper methods when possible.
   */
  getMonacoEditor(): monaco.editor.IStandaloneCodeEditor {
    return this.editor;
  }

  /**
   * Set whether the editor is read-only.
   */
  setReadOnly(readOnly: boolean): void {
    this.editor.updateOptions({ readOnly });
  }

  /**
   * Reveal a specific line in the editor.
   *
   * @param line - The line number to reveal (1-indexed)
   */
  revealLine(line: number): void {
    this.editor.revealLineInCenter(line);
  }

  /**
   * Set the cursor position.
   *
   * @param line - Line number (1-indexed)
   * @param column - Column number (1-indexed)
   */
  setCursorPosition(line: number, column: number): void {
    this.editor.setPosition({ lineNumber: line, column });
    this.editor.revealPositionInCenter({ lineNumber: line, column });
  }

  /**
   * Dispose of the editor and release resources.
   *
   * Call this when the editor is no longer needed.
   */
  dispose(): void {
    // Clear any pending timeout
    if (this.debounceTimeout !== null) {
      clearTimeout(this.debounceTimeout);
      this.debounceTimeout = null;
    }

    // Dispose the Monaco editor
    this.editor.dispose();
  }
}
