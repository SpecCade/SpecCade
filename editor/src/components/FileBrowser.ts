/**
 * File browser sidebar component for navigating spec files.
 *
 * Displays a tree view of directories and spec files (.star, .json),
 * with icons based on detected asset types.
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

/**
 * A file or directory entry from the backend.
 */
interface FileEntry {
  /** Full path to the file or directory */
  path: string;
  /** Display name (filename without path) */
  name: string;
  /** Whether this entry is a directory */
  is_dir: boolean;
  /** Detected asset type for spec files */
  asset_type?: string;
}

/**
 * Icon mapping for different asset types and file/folder states.
 */
const ASSET_ICONS: Record<string, string> = {
  audio: "\uD83D\uDD0A",
  music: "\uD83C\uDFB5",
  texture: "\uD83C\uDFA8",
  static_mesh: "\uD83D\uDCE6",
  mesh: "\uD83D\uDCE6",
  skeletal_mesh: "\uD83E\uDDCD",
  skeletal_animation: "\uD83C\uDFC3",
  folder: "\uD83D\uDCC1",
  file: "\uD83D\uDCC4",
};

/**
 * Get the appropriate icon for a file entry.
 */
function getIcon(entry: FileEntry): string {
  if (entry.is_dir) {
    return ASSET_ICONS.folder;
  }
  if (entry.asset_type && ASSET_ICONS[entry.asset_type]) {
    return ASSET_ICONS[entry.asset_type];
  }
  return ASSET_ICONS.file;
}

/**
 * File browser sidebar component.
 *
 * Provides folder navigation and file selection for loading spec files
 * into the editor.
 */
export class FileBrowser {
  private container: HTMLElement;
  private onFileSelect: (path: string, content: string) => void;
  private wrapper: HTMLDivElement;
  private header: HTMLDivElement;
  private fileList: HTMLDivElement;
  private currentPath: string | null = null;
  private pathHistory: string[] = [];

  /**
   * Create a new file browser.
   *
   * @param container - The HTML element to render into
   * @param onFileSelect - Callback when a file is selected (receives path and content)
   */
  constructor(
    container: HTMLElement,
    onFileSelect: (path: string, content: string) => void
  ) {
    this.container = container;
    this.onFileSelect = onFileSelect;

    // Create wrapper
    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      overflow: hidden;
    `;

    // Create header with title and open folder button
    this.header = this.createHeader();
    this.wrapper.appendChild(this.header);

    // Create file list container
    this.fileList = document.createElement("div");
    this.fileList.style.cssText = `
      flex: 1;
      overflow-y: auto;
      padding: 4px 0;
    `;
    this.wrapper.appendChild(this.fileList);

    // Render initial empty state
    this.renderEmptyState();

    this.container.appendChild(this.wrapper);
  }

  /**
   * Create the header section with the Open Folder button.
   */
  private createHeader(): HTMLDivElement {
    const header = document.createElement("div");
    header.style.cssText = `
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 12px;
      background: #252526;
      border-bottom: 1px solid #333;
    `;

    // Title
    const title = document.createElement("span");
    title.textContent = "Files";
    title.style.cssText = `
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      color: #999;
      letter-spacing: 0.5px;
    `;
    header.appendChild(title);

    // Open Folder button
    const openButton = document.createElement("button");
    openButton.textContent = "Open Folder";
    openButton.style.cssText = `
      padding: 4px 8px;
      font-size: 11px;
      background: #0e639c;
      color: #fff;
      border: none;
      border-radius: 3px;
      cursor: pointer;
      transition: background 0.15s ease;
    `;
    openButton.addEventListener("mouseenter", () => {
      openButton.style.background = "#1177bb";
    });
    openButton.addEventListener("mouseleave", () => {
      openButton.style.background = "#0e639c";
    });
    openButton.addEventListener("click", () => {
      this.openFolder();
    });
    header.appendChild(openButton);

    return header;
  }

  /**
   * Render the empty state when no folder is open.
   */
  private renderEmptyState(): void {
    this.fileList.innerHTML = "";
    const emptyState = document.createElement("div");
    emptyState.style.cssText = `
      padding: 24px 12px;
      text-align: center;
      color: #666;
      font-size: 12px;
    `;
    emptyState.textContent = "No folder open";
    this.fileList.appendChild(emptyState);
  }

  /**
   * Open a folder using the system file picker.
   */
  async openFolder(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        this.pathHistory = [];
        await this.loadFolder(selected);
      }
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  }

  /**
   * Load and display the contents of a folder.
   */
  async loadFolder(path: string): Promise<void> {
    try {
      const entries = await invoke<FileEntry[]>("plugin:speccade|open_folder", {
        path,
      });
      this.currentPath = path;
      this.renderEntries(entries);
    } catch (error) {
      console.error("Failed to load folder:", error);
      this.fileList.innerHTML = "";
      const errorDiv = document.createElement("div");
      errorDiv.style.cssText = `
        padding: 12px;
        color: #f48771;
        font-size: 12px;
      `;
      errorDiv.textContent = `Error: ${String(error)}`;
      this.fileList.appendChild(errorDiv);
    }
  }

  /**
   * Render file entries in the list.
   */
  private renderEntries(entries: FileEntry[]): void {
    this.fileList.innerHTML = "";

    // Add parent directory navigation if we have history
    if (this.pathHistory.length > 0) {
      const backItem = this.createBackItem();
      this.fileList.appendChild(backItem);
    }

    // Render each entry
    for (const entry of entries) {
      const item = this.createEntryItem(entry);
      this.fileList.appendChild(item);
    }

    // Show empty message if no entries
    if (entries.length === 0 && this.pathHistory.length === 0) {
      const emptyDiv = document.createElement("div");
      emptyDiv.style.cssText = `
        padding: 12px;
        color: #666;
        font-size: 12px;
      `;
      emptyDiv.textContent = "No spec files found";
      this.fileList.appendChild(emptyDiv);
    }
  }

  /**
   * Create a back navigation item.
   */
  private createBackItem(): HTMLDivElement {
    const item = document.createElement("div");
    item.style.cssText = `
      display: flex;
      align-items: center;
      padding: 4px 12px;
      cursor: pointer;
      color: #ccc;
      font-size: 13px;
      transition: background 0.1s ease;
    `;

    // Icon
    const icon = document.createElement("span");
    icon.textContent = "\u2190";
    icon.style.cssText = `
      margin-right: 8px;
      font-size: 14px;
    `;
    item.appendChild(icon);

    // Label
    const label = document.createElement("span");
    label.textContent = "..";
    label.style.color = "#999";
    item.appendChild(label);

    // Hover effect
    item.addEventListener("mouseenter", () => {
      item.style.background = "#2a2d2e";
    });
    item.addEventListener("mouseleave", () => {
      item.style.background = "transparent";
    });

    // Click handler - go back to parent
    item.addEventListener("click", () => {
      const parentPath = this.pathHistory.pop();
      if (parentPath) {
        this.loadFolder(parentPath);
      }
    });

    return item;
  }

  /**
   * Create a list item for a file or directory entry.
   */
  private createEntryItem(entry: FileEntry): HTMLDivElement {
    const item = document.createElement("div");
    item.style.cssText = `
      display: flex;
      align-items: center;
      padding: 4px 12px;
      cursor: pointer;
      color: #ccc;
      font-size: 13px;
      transition: background 0.1s ease;
    `;

    // Icon
    const icon = document.createElement("span");
    icon.textContent = getIcon(entry);
    icon.style.cssText = `
      margin-right: 8px;
      font-size: 14px;
    `;
    item.appendChild(icon);

    // Name
    const name = document.createElement("span");
    name.textContent = entry.name;
    name.style.cssText = `
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    `;
    if (entry.is_dir) {
      name.style.color = "#e8c574";
    }
    item.appendChild(name);

    // Hover effect
    item.addEventListener("mouseenter", () => {
      item.style.background = "#2a2d2e";
    });
    item.addEventListener("mouseleave", () => {
      item.style.background = "transparent";
    });

    // Click handler
    item.addEventListener("click", async () => {
      if (entry.is_dir) {
        // Navigate into directory
        if (this.currentPath) {
          this.pathHistory.push(this.currentPath);
        }
        await this.loadFolder(entry.path);
      } else {
        // Open file
        await this.selectFile(entry.path);
      }
    });

    return item;
  }

  /**
   * Select and load a file.
   */
  private async selectFile(path: string): Promise<void> {
    try {
      const content = await invoke<string>("plugin:speccade|read_file", {
        path,
      });
      this.onFileSelect(path, content);
    } catch (error) {
      console.error("Failed to read file:", error);
    }
  }

  /**
   * Get the currently open folder path.
   */
  getCurrentPath(): string | null {
    return this.currentPath;
  }

  /**
   * Refresh the current folder view.
   */
  async refresh(): Promise<void> {
    if (this.currentPath) {
      await this.loadFolder(this.currentPath);
    }
  }

  /**
   * Dispose of the file browser and clear the container.
   */
  dispose(): void {
    this.container.innerHTML = "";
  }
}
