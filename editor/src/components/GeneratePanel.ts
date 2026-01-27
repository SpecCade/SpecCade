/**
 * Generate panel component for asset generation.
 *
 * Provides UI for selecting output directory and generating assets
 * from the current spec.
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

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

/**
 * Result from the generate_full command.
 */
interface GenerateResult {
  success: boolean;
  outputs: GeneratedFile[];
  elapsed_ms: number;
  error?: string;
  lint?: LintOutput;
}

/**
 * A generated file entry.
 */
interface GeneratedFile {
  path: string;
  size_bytes: number;
  format: string;
}

/**
 * Format bytes to human-readable string.
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * Generate panel component.
 *
 * Provides UI for selecting output directory and generating assets.
 */
export class GeneratePanel {
  private container: HTMLElement;
  private getSource: () => string;
  private getFilename: () => string;
  private onLint?: (lint: LintOutput | undefined) => void;
  private wrapper: HTMLDivElement;
  private pathDisplay: HTMLSpanElement;
  private generateButton: HTMLButtonElement;
  private resultsDiv: HTMLDivElement;
  private outputDir: string | null = null;

  /**
   * Create a new generate panel.
   *
   * @param container - The HTML element to render into
   * @param getSource - Callback to get the current editor source
   * @param getFilename - Callback to get the current filename
   * @param onLint - Optional callback invoked with lint results after generation
   */
  constructor(
    container: HTMLElement,
    getSource: () => string,
    getFilename: () => string,
    onLint?: (lint: LintOutput | undefined) => void,
  ) {
    this.container = container;
    this.getSource = getSource;
    this.getFilename = getFilename;
    this.onLint = onLint;

    // Create wrapper
    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      padding: 12px;
      box-sizing: border-box;
      gap: 12px;
    `;

    // Create output directory row
    const outputRow = document.createElement("div");
    outputRow.style.cssText = `
      display: flex;
      align-items: center;
      gap: 8px;
    `;

    const outputLabel = document.createElement("label");
    outputLabel.textContent = "Output:";
    outputLabel.style.cssText = `
      font-size: 12px;
      color: #999;
      white-space: nowrap;
    `;
    outputRow.appendChild(outputLabel);

    this.pathDisplay = document.createElement("span");
    this.pathDisplay.textContent = "Not selected";
    this.pathDisplay.style.cssText = `
      flex: 1;
      font-size: 12px;
      color: #ccc;
      background: #2a2a2a;
      padding: 6px 8px;
      border-radius: 4px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    `;
    outputRow.appendChild(this.pathDisplay);

    const browseButton = document.createElement("button");
    browseButton.textContent = "Browse";
    browseButton.style.cssText = `
      padding: 6px 12px;
      background: #444;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
    `;
    browseButton.addEventListener("mouseenter", () => {
      browseButton.style.background = "#555";
    });
    browseButton.addEventListener("mouseleave", () => {
      browseButton.style.background = "#444";
    });
    browseButton.addEventListener("click", () => this.selectOutputDir());
    outputRow.appendChild(browseButton);

    this.wrapper.appendChild(outputRow);

    // Create generate button
    this.generateButton = document.createElement("button");
    this.generateButton.textContent = "Generate Assets";
    this.generateButton.style.cssText = `
      padding: 10px 16px;
      background: #007acc;
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 13px;
      font-weight: 500;
    `;
    this.generateButton.addEventListener("mouseenter", () => {
      if (!this.generateButton.disabled) {
        this.generateButton.style.background = "#0098ff";
      }
    });
    this.generateButton.addEventListener("mouseleave", () => {
      if (!this.generateButton.disabled) {
        this.generateButton.style.background = "#007acc";
      }
    });
    this.generateButton.addEventListener("click", () => this.generate());
    this.wrapper.appendChild(this.generateButton);

    // Create results div
    this.resultsDiv = document.createElement("div");
    this.resultsDiv.style.cssText = `
      flex: 1;
      overflow-y: auto;
      font-size: 12px;
      color: #ccc;
    `;
    this.wrapper.appendChild(this.resultsDiv);

    this.container.appendChild(this.wrapper);
  }

  /**
   * Open directory picker to select output directory.
   */
  async selectOutputDir(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        this.outputDir = selected;
        this.pathDisplay.textContent = selected;
        this.pathDisplay.title = selected;
      }
    } catch (error) {
      console.error("Failed to select output directory:", error);
    }
  }

  /**
   * Generate assets from the current spec.
   */
  async generate(): Promise<void> {
    // If no output dir selected, prompt user to select one
    if (!this.outputDir) {
      await this.selectOutputDir();
      if (!this.outputDir) {
        return; // User cancelled
      }
    }

    const source = this.getSource();
    if (!source.trim()) {
      this.showError("No spec content to generate");
      return;
    }

    // Disable button and show generating state
    this.generateButton.disabled = true;
    this.generateButton.textContent = "Generating...";
    this.generateButton.style.background = "#555";
    this.generateButton.style.cursor = "wait";
    this.resultsDiv.innerHTML = "";

    try {
      const result = await invoke<GenerateResult>("plugin:speccade|generate_full", {
        source,
        filename: this.getFilename(),
        outputDir: this.outputDir,
      });

      if (result.success) {
        this.showSuccess(result);
        this.onLint?.(result.lint);
      } else {
        this.showError(result.error ?? "Unknown error");
        this.onLint?.(undefined);
      }
    } catch (error) {
      this.showError(String(error));
    } finally {
      // Re-enable button
      this.generateButton.disabled = false;
      this.generateButton.textContent = "Generate Assets";
      this.generateButton.style.background = "#007acc";
      this.generateButton.style.cursor = "pointer";
    }
  }

  /**
   * Show successful generation results.
   */
  private showSuccess(result: GenerateResult): void {
    this.resultsDiv.innerHTML = "";

    // Success header
    const header = document.createElement("div");
    header.style.cssText = `
      padding: 8px 12px;
      background: #1e3a1e;
      border-radius: 4px;
      margin-bottom: 8px;
      color: #4ec94e;
    `;
    header.textContent = `Generated ${result.outputs.length} file${result.outputs.length === 1 ? "" : "s"} in ${result.elapsed_ms}ms`;
    this.resultsDiv.appendChild(header);

    // File list
    if (result.outputs.length > 0) {
      const fileList = document.createElement("div");
      fileList.style.cssText = `
        display: flex;
        flex-direction: column;
        gap: 4px;
      `;

      for (const file of result.outputs) {
        const fileItem = document.createElement("div");
        fileItem.style.cssText = `
          display: flex;
          justify-content: space-between;
          padding: 6px 8px;
          background: #2a2a2a;
          border-radius: 3px;
        `;

        const fileName = document.createElement("span");
        fileName.textContent = file.path;
        fileName.style.cssText = `
          flex: 1;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        `;
        fileName.title = file.path;
        fileItem.appendChild(fileName);

        const fileSize = document.createElement("span");
        fileSize.textContent = formatBytes(file.size_bytes);
        fileSize.style.cssText = `
          margin-left: 8px;
          color: #888;
          white-space: nowrap;
        `;
        fileItem.appendChild(fileSize);

        fileList.appendChild(fileItem);
      }

      this.resultsDiv.appendChild(fileList);
    }

    // Show lint summary if there are issues
    if (result.lint && result.lint.issues.length > 0) {
      const lintHeader = document.createElement("div");
      const hasErrors = result.lint.error_count > 0;
      lintHeader.style.cssText = `
        padding: 8px 12px;
        background: ${hasErrors ? "#3a2a1e" : "#3a3a1e"};
        border-radius: 4px;
        margin-top: 8px;
        color: ${hasErrors ? "#f4a871" : "#e4c94e"};
        font-size: 12px;
      `;
      const parts: string[] = [];
      if (result.lint.error_count > 0) parts.push(`${result.lint.error_count} error${result.lint.error_count === 1 ? "" : "s"}`);
      if (result.lint.warning_count > 0) parts.push(`${result.lint.warning_count} warning${result.lint.warning_count === 1 ? "" : "s"}`);
      if (result.lint.info_count > 0) parts.push(`${result.lint.info_count} info`);
      lintHeader.textContent = `Lint: ${parts.join(", ")}`;
      this.resultsDiv.appendChild(lintHeader);

      // Show individual issues
      const issueList = document.createElement("div");
      issueList.style.cssText = `
        display: flex;
        flex-direction: column;
        gap: 3px;
        margin-top: 4px;
      `;
      for (const issue of result.lint.issues) {
        const issueEl = document.createElement("div");
        const color = issue.severity === "error" ? "#f48771" : issue.severity === "warning" ? "#e4c94e" : "#9cdcfe";
        const badge = issue.severity === "error" ? "E" : issue.severity === "warning" ? "W" : "I";
        issueEl.style.cssText = `
          display: flex;
          align-items: flex-start;
          gap: 6px;
          padding: 4px 8px;
          font-size: 11px;
          color: ${color};
        `;
        const specPath = issue.spec_path ? ` <span style="opacity:0.7">[${issue.spec_path}]</span>` : "";
        issueEl.innerHTML = `<span style="font-weight:700;flex:0 0 auto">${badge}</span><span>${issue.rule_id}: ${issue.message}${specPath}${issue.suggestion ? ". " + issue.suggestion : ""}</span>`;
        issueList.appendChild(issueEl);
      }
      this.resultsDiv.appendChild(issueList);
    }
  }

  /**
   * Show an error message.
   */
  private showError(message: string): void {
    this.resultsDiv.innerHTML = "";

    const errorDiv = document.createElement("div");
    errorDiv.style.cssText = `
      padding: 8px 12px;
      background: #3a1e1e;
      border-radius: 4px;
      color: #f48771;
    `;
    errorDiv.textContent = `Error: ${message}`;
    this.resultsDiv.appendChild(errorDiv);
  }

  /**
   * Dispose of the panel and clear the container.
   */
  dispose(): void {
    this.container.innerHTML = "";
  }
}
