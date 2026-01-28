/**
 * Tree view component for hierarchical file navigation.
 *
 * Displays an expandable tree of files and directories with virtual scrolling
 * for performance. Supports filtering, expansion state management, and active
 * file highlighting.
 */

/**
 * A file or directory node in the tree.
 */
export interface FileNode {
  /** Display name of the file/directory */
  name: string;
  /** Path relative to project root */
  path: string;
  /** Whether this is a directory */
  is_dir: boolean;
  /** Detected asset type for spec files */
  asset_type?: string;
  /** Child nodes (for directories) */
  children?: FileNode[];
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
 * Get the appropriate icon for a file node.
 */
function getIcon(node: FileNode): string {
  if (node.is_dir) {
    return ASSET_ICONS.folder;
  }
  if (node.asset_type && ASSET_ICONS[node.asset_type]) {
    return ASSET_ICONS[node.asset_type];
  }
  return ASSET_ICONS.file;
}

/**
 * A flattened tree row for rendering.
 */
interface TreeRow {
  node: FileNode;
  depth: number;
  isExpanded: boolean;
}

const ROW_HEIGHT = 24;
const BUFFER_ROWS = 10;

/**
 * Tree view component with virtual scrolling.
 *
 * Provides an expandable tree view of files and directories with efficient
 * virtual scrolling for large directory structures.
 */
export class FileTree {
  private container: HTMLElement;
  private onFileSelect: (path: string) => void;
  private wrapper: HTMLDivElement;
  private scrollContainer: HTMLDivElement;
  private spacer: HTMLDivElement;
  private rowsContainer: HTMLDivElement;

  private tree: FileNode[] = [];
  private expandedPaths: Set<string> = new Set();
  private activePath: string | null = null;
  private filterPredicate: ((node: FileNode) => boolean) | null = null;
  private visibleRows: TreeRow[] = [];
  private focusIndex: number = -1;
  private onToggleSelect: ((path: string) => void) | null = null;
  private typeAheadBuffer: string = "";
  private typeAheadTimer: ReturnType<typeof setTimeout> | null = null;
  private keydownHandler: ((e: KeyboardEvent) => void) | null = null;
  private scrollHandler: (() => void) | null = null;

  /**
   * Create a new file tree.
   *
   * @param container - The HTML element to render into
   * @param onFileSelect - Callback when a file is selected (receives path)
   */
  constructor(container: HTMLElement, onFileSelect: (path: string) => void) {
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
      background: #1a1a1a;
    `;

    // Create scroll container
    this.scrollContainer = document.createElement("div");
    this.scrollContainer.style.cssText = `
      flex: 1;
      overflow-y: auto;
      overflow-x: hidden;
      position: relative;
    `;

    // Create spacer for virtual scrolling
    this.spacer = document.createElement("div");
    this.spacer.style.cssText = `
      width: 100%;
      pointer-events: none;
    `;
    this.scrollContainer.appendChild(this.spacer);

    // Create rows container
    this.rowsContainer = document.createElement("div");
    this.rowsContainer.style.cssText = `
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
    `;
    this.scrollContainer.appendChild(this.rowsContainer);

    // Handle scroll events
    this.scrollHandler = () => this.renderVisibleRows();
    this.scrollContainer.addEventListener("scroll", this.scrollHandler);

    this.wrapper.appendChild(this.scrollContainer);
    this.container.appendChild(this.wrapper);

    // Make focusable for keyboard navigation
    this.wrapper.tabIndex = 0;
    this.wrapper.style.outline = "none";
    this.keydownHandler = (e: KeyboardEvent) => this.handleKeydown(e);
    this.wrapper.addEventListener("keydown", this.keydownHandler);
  }

  /**
   * Set the tree data.
   */
  setTree(nodes: FileNode[]): void {
    this.tree = nodes;
    this.rebuildVisibleRows();
  }

  /**
   * Set the active file path for highlighting.
   */
  setActiveFile(path: string | null): void {
    this.activePath = path;
    this.renderVisibleRows();
  }

  /**
   * Get the currently expanded folder paths.
   */
  getExpandedPaths(): Set<string> {
    return new Set(this.expandedPaths);
  }

  /**
   * Set the expanded folder paths.
   */
  setExpandedPaths(paths: Set<string>): void {
    this.expandedPaths = new Set(paths);
    this.rebuildVisibleRows();
  }

  /**
   * Set a filter predicate for visible nodes.
   *
   * Folders are visible if any descendant matches the predicate.
   */
  setFilter(predicate: ((node: FileNode) => boolean) | null): void {
    this.filterPredicate = predicate;
    this.rebuildVisibleRows();
  }

  /**
   * Refresh the tree view with current data.
   */
  refresh(): void {
    this.rebuildVisibleRows();
  }

  /**
   * Set a callback for toggling selection (Space key).
   */
  setOnToggleSelect(handler: (path: string) => void): void {
    this.onToggleSelect = handler;
  }

  /**
   * Focus the tree for keyboard navigation.
   */
  focus(): void {
    this.wrapper.focus();
  }

  /**
   * Handle keyboard events for tree navigation.
   */
  private handleKeydown(e: KeyboardEvent): void {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        this.moveFocus(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        this.moveFocus(-1);
        break;
      case "ArrowRight":
        e.preventDefault();
        if (this.focusIndex >= 0 && this.focusIndex < this.visibleRows.length) {
          const row = this.visibleRows[this.focusIndex];
          if (row.node.is_dir && !row.isExpanded) {
            this.toggleExpanded(row.node.path);
          }
        }
        break;
      case "ArrowLeft":
        e.preventDefault();
        if (this.focusIndex >= 0 && this.focusIndex < this.visibleRows.length) {
          const row = this.visibleRows[this.focusIndex];
          if (row.node.is_dir && row.isExpanded) {
            this.toggleExpanded(row.node.path);
          }
        }
        break;
      case "Enter":
        e.preventDefault();
        if (this.focusIndex >= 0 && this.focusIndex < this.visibleRows.length) {
          const row = this.visibleRows[this.focusIndex];
          if (row.node.is_dir) {
            this.toggleExpanded(row.node.path);
          } else {
            this.onFileSelect(row.node.path);
          }
        }
        break;
      case " ":
        e.preventDefault();
        if (this.focusIndex >= 0 && this.focusIndex < this.visibleRows.length) {
          const row = this.visibleRows[this.focusIndex];
          if (!row.node.is_dir && this.onToggleSelect) {
            this.onToggleSelect(row.node.path);
          }
        }
        break;
      case "Home":
        e.preventDefault();
        this.setFocusIndex(0);
        break;
      case "End":
        e.preventDefault();
        this.setFocusIndex(this.visibleRows.length - 1);
        break;
      default:
        // Type-ahead: single printable character jumps to matching filename
        if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
          this.handleTypeAhead(e.key);
        }
        break;
    }
  }

  /**
   * Move focus by a delta amount.
   */
  private moveFocus(delta: number): void {
    const next = Math.max(0, Math.min(this.visibleRows.length - 1, this.focusIndex + delta));
    this.setFocusIndex(next);
  }

  /**
   * Set focus to a specific row index and scroll it into view.
   */
  private setFocusIndex(index: number): void {
    if (index < 0 || index >= this.visibleRows.length) return;
    this.focusIndex = index;
    // Scroll into view
    const rowTop = index * ROW_HEIGHT;
    const rowBottom = rowTop + ROW_HEIGHT;
    const scrollTop = this.scrollContainer.scrollTop;
    const viewHeight = this.scrollContainer.clientHeight;
    if (rowTop < scrollTop) {
      this.scrollContainer.scrollTop = rowTop;
    } else if (rowBottom > scrollTop + viewHeight) {
      this.scrollContainer.scrollTop = rowBottom - viewHeight;
    }
    this.renderVisibleRows();
  }

  /**
   * Handle type-ahead character input for jumping to files.
   */
  private handleTypeAhead(char: string): void {
    this.typeAheadBuffer += char.toLowerCase();
    if (this.typeAheadTimer) clearTimeout(this.typeAheadTimer);
    this.typeAheadTimer = setTimeout(() => {
      this.typeAheadBuffer = "";
    }, 500);

    const query = this.typeAheadBuffer;
    for (let i = 0; i < this.visibleRows.length; i++) {
      if (this.visibleRows[i].node.name.toLowerCase().startsWith(query)) {
        this.setFocusIndex(i);
        break;
      }
    }
  }

  /**
   * Dispose of the file tree and clear the container.
   */
  dispose(): void {
    if (this.keydownHandler) {
      this.wrapper.removeEventListener("keydown", this.keydownHandler);
    }
    if (this.scrollHandler) {
      this.scrollContainer.removeEventListener("scroll", this.scrollHandler);
    }
    if (this.typeAheadTimer) clearTimeout(this.typeAheadTimer);
    this.container.innerHTML = "";
  }

  /**
   * Rebuild the flat list of visible rows from the tree.
   */
  private rebuildVisibleRows(): void {
    this.visibleRows = [];
    this.flattenTree(this.tree, 0);

    // Update spacer height
    const totalHeight = this.visibleRows.length * ROW_HEIGHT;
    this.spacer.style.height = `${totalHeight}px`;

    this.renderVisibleRows();
  }

  /**
   * Recursively flatten the tree into visible rows.
   */
  private flattenTree(nodes: FileNode[], depth: number): void {
    for (const node of nodes) {
      // Check if node should be visible based on filter
      if (this.filterPredicate && !this.shouldShowNode(node)) {
        continue;
      }

      const isExpanded = this.expandedPaths.has(node.path);
      this.visibleRows.push({ node, depth, isExpanded });

      // If directory is expanded, recurse into children
      if (node.is_dir && isExpanded && node.children) {
        this.flattenTree(node.children, depth + 1);
      }
    }
  }

  /**
   * Check if a node should be shown based on the filter predicate.
   *
   * A node is shown if:
   * - It matches the predicate, OR
   * - It's a directory and any descendant matches the predicate
   */
  private shouldShowNode(node: FileNode): boolean {
    if (!this.filterPredicate) {
      return true;
    }

    // Check if node itself matches
    if (this.filterPredicate(node)) {
      return true;
    }

    // If it's a directory, check if any descendant matches
    if (node.is_dir && node.children) {
      return this.hasMatchingDescendant(node.children);
    }

    return false;
  }

  /**
   * Check if any descendant of the given nodes matches the filter.
   */
  private hasMatchingDescendant(nodes: FileNode[]): boolean {
    for (const node of nodes) {
      if (this.filterPredicate && this.filterPredicate(node)) {
        return true;
      }
      if (node.is_dir && node.children) {
        if (this.hasMatchingDescendant(node.children)) {
          return true;
        }
      }
    }
    return false;
  }

  /**
   * Render only the rows visible in the current viewport.
   */
  private renderVisibleRows(): void {
    const scrollTop = this.scrollContainer.scrollTop;
    const viewportHeight = this.scrollContainer.clientHeight;

    // Calculate visible range with buffer
    const startIndex = Math.max(
      0,
      Math.floor(scrollTop / ROW_HEIGHT) - BUFFER_ROWS
    );
    const endIndex = Math.min(
      this.visibleRows.length,
      Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + BUFFER_ROWS
    );

    // Clear existing rows
    this.rowsContainer.innerHTML = "";

    // Render visible rows
    for (let i = startIndex; i < endIndex; i++) {
      const row = this.visibleRows[i];
      const rowElement = this.createRowElement(row, i);
      this.rowsContainer.appendChild(rowElement);
    }
  }

  /**
   * Create a DOM element for a tree row.
   */
  private createRowElement(row: TreeRow, index: number): HTMLDivElement {
    const rowElement = document.createElement("div");
    const isActive = this.activePath === row.node.path;
    const isFocused = this.focusIndex === index;
    const paddingLeft = row.depth * 16;
    const bg = isActive ? "#094771" : isFocused ? "#2a2d2e" : "transparent";
    const outline = isFocused ? "1px solid #007acc" : "none";

    rowElement.style.cssText = `
      position: absolute;
      top: ${index * ROW_HEIGHT}px;
      left: 0;
      right: 0;
      height: ${ROW_HEIGHT}px;
      display: flex;
      align-items: center;
      padding: 4px 12px;
      padding-left: ${paddingLeft + 12}px;
      cursor: pointer;
      color: #ccc;
      font-size: 13px;
      transition: background 0.1s ease;
      background: ${bg};
      outline: ${outline};
      outline-offset: -1px;
    `;

    // Toggle icon for directories
    if (row.node.is_dir) {
      const toggle = document.createElement("span");
      toggle.textContent = row.isExpanded ? "\u25BC" : "\u25B6";
      toggle.style.cssText = `
        margin-right: 4px;
        font-size: 10px;
        color: #999;
      `;
      rowElement.appendChild(toggle);
    } else {
      // Spacer for files to align with expanded directories
      const spacer = document.createElement("span");
      spacer.style.cssText = `
        width: 14px;
        display: inline-block;
      `;
      rowElement.appendChild(spacer);
    }

    // Icon
    const icon = document.createElement("span");
    icon.textContent = getIcon(row.node);
    icon.style.cssText = `
      margin-right: 8px;
      font-size: 14px;
    `;
    rowElement.appendChild(icon);

    // Name
    const name = document.createElement("span");
    name.textContent = row.node.name;
    name.style.cssText = `
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    `;
    if (row.node.is_dir) {
      name.style.color = "#e8c574";
    }
    rowElement.appendChild(name);

    // Hover effect (only if not active)
    if (!isActive) {
      rowElement.addEventListener("mouseenter", () => {
        rowElement.style.background = "#2a2d2e";
      });
      rowElement.addEventListener("mouseleave", () => {
        rowElement.style.background = isFocused ? "#2a2d2e" : "transparent";
      });
    }

    // Click handler
    rowElement.addEventListener("click", () => {
      if (row.node.is_dir) {
        this.toggleExpanded(row.node.path);
      } else {
        this.onFileSelect(row.node.path);
      }
    });

    return rowElement;
  }

  /**
   * Toggle the expanded state of a directory.
   */
  private toggleExpanded(path: string): void {
    if (this.expandedPaths.has(path)) {
      this.expandedPaths.delete(path);
    } else {
      this.expandedPaths.add(path);
    }
    this.rebuildVisibleRows();
  }
}
