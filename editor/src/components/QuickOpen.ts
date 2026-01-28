/**
 * Quick Open dialog component.
 *
 * VS Code-style Ctrl+P command palette for quickly navigating to files.
 * Features fuzzy search with keyboard navigation and instant file selection.
 */

import { fuzzyMatch } from "./SearchBar";

/**
 * File entry for quick open list.
 */
interface FileEntry {
  path: string;
  name: string;
  asset_type?: string;
}

/**
 * File entry with fuzzy match result.
 */
interface ScoredEntry {
  entry: FileEntry;
  score: number;
  indices: number[];
}

/**
 * Asset type emoji mapping.
 */
const ASSET_ICONS: Record<string, string> = {
  audio: "🔊",
  music: "🎵",
  texture: "🎨",
  mesh: "📦",
  static_mesh: "📦",
  skeletal_mesh: "🧍",
  skeletal_animation: "🏃",
};

const DEFAULT_ICON = "📄";

/**
 * Get the appropriate icon for an asset type.
 */
function getIcon(assetType: string | undefined): string {
  if (!assetType) {
    return DEFAULT_ICON;
  }
  return ASSET_ICONS[assetType] || DEFAULT_ICON;
}

/**
 * VS Code-style quick open dialog (Ctrl+P).
 *
 * Displays a centered overlay with fuzzy search for quickly navigating
 * to files. Supports keyboard navigation (Arrow Up/Down, Enter, Escape).
 */
export class QuickOpen {
  private overlay: HTMLDivElement | null = null;
  private dialog: HTMLDivElement | null = null;
  private input: HTMLInputElement | null = null;
  private resultsList: HTMLDivElement | null = null;
  private onSelect: (path: string) => void;
  private keydownHandler: ((e: KeyboardEvent) => void) | null = null;

  private files: FileEntry[] = [];
  private filteredResults: ScoredEntry[] = [];
  private focusedIndex = 0;

  private readonly MAX_RESULTS = 20;

  /**
   * Create a new QuickOpen dialog.
   *
   * @param onSelect - Callback when a file is selected (receives path)
   */
  constructor(onSelect: (path: string) => void) {
    this.onSelect = onSelect;
  }

  /**
   * Show the quick open dialog with the given file list.
   *
   * @param files - Array of file entries to search through
   */
  show(files: FileEntry[]): void {
    this.files = files;
    this.filteredResults = [];
    this.focusedIndex = 0;

    // Create overlay
    this.overlay = document.createElement("div");
    this.overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.5);
      display: flex;
      align-items: flex-start;
      justify-content: center;
      z-index: 1000;
      padding-top: 20vh;
    `;

    // Close on backdrop click
    this.overlay.addEventListener("click", (e) => {
      if (e.target === this.overlay) {
        this.hide();
      }
    });

    // Create dialog
    this.dialog = document.createElement("div");
    this.dialog.style.cssText = `
      background: #252526;
      max-width: 500px;
      width: 90%;
      border: 1px solid #333;
      border-radius: 8px;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
      display: flex;
      flex-direction: column;
      overflow: hidden;
    `;

    // Create input
    this.input = document.createElement("input");
    this.input.type = "text";
    this.input.placeholder = "Go to file...";
    this.input.style.cssText = `
      width: 100%;
      background: #3c3c3c;
      border: none;
      border-bottom: 1px solid #333;
      color: #ccc;
      padding: 12px 16px;
      font-size: 14px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      outline: none;
      box-sizing: border-box;
    `;

    // Create results list
    this.resultsList = document.createElement("div");
    this.resultsList.style.cssText = `
      max-height: 300px;
      overflow-y: auto;
      overflow-x: hidden;
    `;

    // Handle input changes
    this.input.addEventListener("input", () => {
      this.updateResults();
    });

    // Assemble dialog
    this.dialog.appendChild(this.input);
    this.dialog.appendChild(this.resultsList);
    this.overlay.appendChild(this.dialog);
    document.body.appendChild(this.overlay);

    // Set up keyboard handler
    this.keydownHandler = (e: KeyboardEvent) => {
      this.handleKeyDown(e);
    };
    document.addEventListener("keydown", this.keydownHandler);

    // Focus input and show all results initially
    this.input.focus();
    this.updateResults();
  }

  /**
   * Hide and remove the quick open dialog.
   */
  hide(): void {
    if (this.keydownHandler) {
      document.removeEventListener("keydown", this.keydownHandler);
      this.keydownHandler = null;
    }
    if (this.overlay) {
      document.body.removeChild(this.overlay);
      this.overlay = null;
      this.dialog = null;
      this.input = null;
      this.resultsList = null;
    }
    this.files = [];
    this.filteredResults = [];
    this.focusedIndex = 0;
  }

  /**
   * Handle keyboard navigation.
   */
  private handleKeyDown(e: KeyboardEvent): void {
    switch (e.key) {
      case "Escape":
        e.preventDefault();
        this.hide();
        break;

      case "ArrowDown":
        e.preventDefault();
        if (this.filteredResults.length > 0) {
          this.focusedIndex = Math.min(
            this.focusedIndex + 1,
            this.filteredResults.length - 1
          );
          this.renderResults();
        }
        break;

      case "ArrowUp":
        e.preventDefault();
        if (this.filteredResults.length > 0) {
          this.focusedIndex = Math.max(this.focusedIndex - 1, 0);
          this.renderResults();
        }
        break;

      case "Enter":
        e.preventDefault();
        if (
          this.filteredResults.length > 0 &&
          this.focusedIndex < this.filteredResults.length
        ) {
          this.selectResult(this.filteredResults[this.focusedIndex].entry);
        }
        break;
    }
  }

  /**
   * Update filtered results based on current input.
   */
  private updateResults(): void {
    if (!this.input) return;

    const query = this.input.value.trim();

    if (!query) {
      // Show all files when no query
      this.filteredResults = this.files.slice(0, this.MAX_RESULTS).map((entry) => ({
        entry,
        score: 0,
        indices: [],
      }));
    } else {
      // Fuzzy match and sort by score
      const scored: ScoredEntry[] = [];

      for (const entry of this.files) {
        const result = fuzzyMatch(query, entry.path);
        if (result.matches) {
          scored.push({
            entry,
            score: result.score,
            indices: result.indices,
          });
        }
      }

      // Sort by score (highest first) and limit to MAX_RESULTS
      scored.sort((a, b) => b.score - a.score);
      this.filteredResults = scored.slice(0, this.MAX_RESULTS);
    }

    // Reset focused index
    this.focusedIndex = 0;

    this.renderResults();
  }

  /**
   * Render the results list.
   */
  private renderResults(): void {
    if (!this.resultsList) return;

    // Clear existing results
    this.resultsList.innerHTML = "";

    if (this.filteredResults.length === 0) {
      const noResults = document.createElement("div");
      noResults.textContent = "No files found";
      noResults.style.cssText = `
        padding: 16px;
        text-align: center;
        color: #666;
        font-size: 13px;
      `;
      this.resultsList.appendChild(noResults);
      return;
    }

    // Render each result
    for (let i = 0; i < this.filteredResults.length; i++) {
      const { entry } = this.filteredResults[i];
      const isFocused = i === this.focusedIndex;

      const resultRow = document.createElement("div");
      resultRow.style.cssText = `
        padding: 8px 16px;
        font-size: 13px;
        color: #ccc;
        cursor: pointer;
        background: ${isFocused ? "#094771" : "transparent"};
        display: flex;
        align-items: center;
        gap: 8px;
      `;

      // Hover effect (only if not focused)
      if (!isFocused) {
        resultRow.addEventListener("mouseenter", () => {
          resultRow.style.background = "#2a2d2e";
        });
        resultRow.addEventListener("mouseleave", () => {
          resultRow.style.background = "transparent";
        });
      }

      // Click handler
      resultRow.addEventListener("click", () => {
        this.selectResult(entry);
      });

      // Icon
      const icon = document.createElement("span");
      icon.textContent = getIcon(entry.asset_type);
      icon.style.cssText = `
        font-size: 14px;
        flex-shrink: 0;
      `;
      resultRow.appendChild(icon);

      // File info container
      const infoContainer = document.createElement("div");
      infoContainer.style.cssText = `
        flex: 1;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        gap: 2px;
      `;

      // File name
      const fileName = document.createElement("div");
      fileName.textContent = entry.name;
      fileName.style.cssText = `
        color: #ccc;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      `;
      infoContainer.appendChild(fileName);

      // File path (dimmer)
      const filePath = document.createElement("div");
      filePath.textContent = entry.path;
      filePath.style.cssText = `
        color: #666;
        font-size: 11px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      `;
      infoContainer.appendChild(filePath);

      resultRow.appendChild(infoContainer);
      this.resultsList.appendChild(resultRow);
    }
  }

  /**
   * Select a result and close the dialog.
   */
  private selectResult(entry: FileEntry): void {
    this.hide();
    this.onSelect(entry.path);
  }
}
