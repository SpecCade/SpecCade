/**
 * ProjectExplorer coordinator component.
 *
 * Replaces FileBrowser with a full-featured project explorer: tree view,
 * search bar, filter bar, multi-select with batch actions, and quick open.
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FileTree, FileNode } from "./FileTree";
import { SearchBar, fuzzyMatch } from "./SearchBar";
import { FilterBar } from "./FilterBar";
import { SelectionManager, BatchActionBar } from "./SelectionManager";
import { QuickOpen } from "./QuickOpen";

/**
 * ProjectExplorer — owns the file data model and coordinates sub-components.
 */
export class ProjectExplorer {
  private container: HTMLElement;
  private onFileSelect: (path: string, content: string) => void;
  private wrapper: HTMLDivElement;

  private projectRoot: string | null = null;
  private tree: FileNode[] = [];

  // Sub-components
  private searchBar: SearchBar | null = null;
  private filterBar: FilterBar | null = null;
  private fileTree: FileTree | null = null;
  private selectionManager: SelectionManager;
  private batchActionBar: BatchActionBar | null = null;
  private quickOpen: QuickOpen;

  // Filter state
  private searchQuery: string = "";
  private activeTypes: Set<string> | null = null;

  // Keyboard handler reference
  private keydownHandler: ((e: KeyboardEvent) => void) | null = null;

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

    // Header with title + Open Folder button
    const header = this.createHeader();
    this.wrapper.appendChild(header);

    // Search bar
    const searchContainer = document.createElement("div");
    this.searchBar = new SearchBar(searchContainer, (query) => {
      this.searchQuery = query;
      this.applyFilters();
    });
    this.wrapper.appendChild(searchContainer);

    // Filter bar
    const filterContainer = document.createElement("div");
    this.filterBar = new FilterBar(filterContainer, (types) => {
      this.activeTypes = types;
      this.applyFilters();
    });
    this.wrapper.appendChild(filterContainer);

    // File tree (takes remaining space)
    const treeContainer = document.createElement("div");
    treeContainer.style.cssText = `
      flex: 1;
      overflow: hidden;
      position: relative;
    `;
    this.fileTree = new FileTree(treeContainer, (path) => {
      this.openFile(path);
    });
    this.wrapper.appendChild(treeContainer);

    // Selection manager
    this.selectionManager = new SelectionManager({
      onSelectionChange: () => {
        this.batchActionBar?.update();
        this.fileTree?.refresh();
      },
      onValidate: async (paths) => {
        console.log("Validate:", paths);
        try {
          const result = await invoke<{
            total: number;
            valid: number;
            invalid: number;
            results: Array<{ path: string; result: { success: boolean } }>;
          }>("plugin:speccade|batch_validate", {
            paths,
            budget: null,
          });
          console.log("Validation result:", result);
          if (result.invalid > 0) {
            alert(`Validation: ${result.valid}/${result.total} passed`);
          } else {
            alert(`All ${result.total} specs valid!`);
          }
        } catch (e) {
          console.error("Validation failed:", e);
          alert(`Validation error: ${e}`);
        }
      },
      onGenerate: (paths) => {
        console.log("Generate:", paths);
        // TODO: wire to backend batch_generate command
      },
      onDelete: (paths) => {
        console.log("Delete:", paths);
        // TODO: wire to backend delete + refresh
      },
    });

    // Wire selection toggle from FileTree
    this.fileTree.setOnToggleSelect((path) => {
      this.selectionManager.toggle(path);
    });

    // Batch action bar
    const batchContainer = document.createElement("div");
    this.batchActionBar = new BatchActionBar(batchContainer, this.selectionManager);
    this.wrapper.appendChild(batchContainer);

    // Quick open (Ctrl+P)
    this.quickOpen = new QuickOpen((path) => {
      this.openFile(path);
    });

    // Register Ctrl+P handler
    this.keydownHandler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "p") {
        e.preventDefault();
        this.showQuickOpen();
      }
    };
    document.addEventListener("keydown", this.keydownHandler);

    this.container.appendChild(this.wrapper);
  }

  /**
   * Create the header with title and Open Folder button.
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

    const title = document.createElement("span");
    title.textContent = "Explorer";
    title.style.cssText = `
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      color: #999;
      letter-spacing: 0.5px;
    `;
    header.appendChild(title);

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
   * Open a folder via system dialog and scan the project tree.
   */
  async openFolder(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        await this.loadProject(selected);
      }
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  }

  /**
   * Load a project from a root path.
   */
  async loadProject(rootPath: string): Promise<void> {
    try {
      const tree = await invoke<FileNode[]>("plugin:speccade|scan_project_tree", {
        path: rootPath,
      });
      this.projectRoot = rootPath;
      this.tree = tree;
      this.selectionManager.clear();
      this.searchQuery = "";
      this.activeTypes = null;
      this.searchBar?.clear();

      this.fileTree?.setTree(tree);
      this.updateCounts();
    } catch (error) {
      console.error("Failed to scan project:", error);
    }
  }

  /**
   * Refresh the project tree (e.g. after file watcher event).
   */
  async refresh(): Promise<void> {
    if (this.projectRoot) {
      const expanded = this.fileTree?.getExpandedPaths();
      await this.loadProject(this.projectRoot);
      if (expanded) {
        this.fileTree?.setExpandedPaths(expanded);
      }
    }
  }

  /**
   * Set the active file for highlighting in the tree.
   */
  setActiveFile(path: string | null): void {
    this.fileTree?.setActiveFile(path);
  }

  /**
   * Get the project root path.
   */
  getProjectRoot(): string | null {
    return this.projectRoot;
  }

  /**
   * Open a file by its relative path.
   */
  private async openFile(relativePath: string): Promise<void> {
    if (!this.projectRoot) return;

    const fullPath = this.projectRoot + "/" + relativePath;
    try {
      const content = await invoke<string>("plugin:speccade|read_file", {
        path: fullPath,
      });
      this.fileTree?.setActiveFile(relativePath);
      this.onFileSelect(fullPath, content);
    } catch (error) {
      console.error("Failed to read file:", error);
    }
  }

  /**
   * Show the Ctrl+P quick open dialog.
   */
  private showQuickOpen(): void {
    const files = this.collectFiles(this.tree);
    this.quickOpen.show(files);
  }

  /**
   * Collect all file entries from the tree (flattened).
   */
  private collectFiles(
    nodes: FileNode[]
  ): Array<{ path: string; name: string; asset_type?: string }> {
    const result: Array<{ path: string; name: string; asset_type?: string }> = [];
    for (const node of nodes) {
      if (node.is_dir && node.children) {
        result.push(...this.collectFiles(node.children));
      } else if (!node.is_dir) {
        result.push({
          path: node.path,
          name: node.name,
          asset_type: node.asset_type,
        });
      }
    }
    return result;
  }

  /**
   * Apply search and filter predicates to the tree.
   */
  private applyFilters(): void {
    const hasSearch = this.searchQuery.length > 0;
    const hasTypeFilter = this.activeTypes !== null;

    if (!hasSearch && !hasTypeFilter) {
      this.fileTree?.setFilter(null);
      return;
    }

    this.fileTree?.setFilter((node: FileNode) => {
      // Directories are handled by FileTree's shouldShowNode (ancestor logic)
      if (node.is_dir) return false;

      // Type filter
      if (hasTypeFilter && this.activeTypes) {
        if (!node.asset_type || !this.activeTypes.has(node.asset_type)) {
          return false;
        }
      }

      // Search filter
      if (hasSearch) {
        const result = fuzzyMatch(this.searchQuery, node.name);
        if (!result.matches) return false;
      }

      return true;
    });

    // Auto-expand folders containing matches when searching
    if (hasSearch) {
      this.autoExpandMatches(this.tree);
    }
  }

  /**
   * Auto-expand folders that contain matching files.
   */
  private autoExpandMatches(nodes: FileNode[]): void {
    for (const node of nodes) {
      if (node.is_dir && node.children) {
        if (this.hasFilterMatch(node.children)) {
          const expanded = this.fileTree?.getExpandedPaths() ?? new Set<string>();
          expanded.add(node.path);
          this.fileTree?.setExpandedPaths(expanded);
        }
        this.autoExpandMatches(node.children);
      }
    }
  }

  /**
   * Check if any descendant matches the current filters.
   */
  private hasFilterMatch(nodes: FileNode[]): boolean {
    for (const node of nodes) {
      if (!node.is_dir) {
        if (this.activeTypes && (!node.asset_type || !this.activeTypes.has(node.asset_type))) {
          continue;
        }
        if (this.searchQuery) {
          const result = fuzzyMatch(this.searchQuery, node.name);
          if (!result.matches) continue;
        }
        return true;
      }
      if (node.children && this.hasFilterMatch(node.children)) {
        return true;
      }
    }
    return false;
  }

  /**
   * Update asset type count badges in the FilterBar.
   */
  private updateCounts(): void {
    const counts = new Map<string, number>();
    this.countTypes(this.tree, counts);
    this.filterBar?.setCounts(counts);
  }

  /**
   * Recursively count asset types in the tree.
   */
  private countTypes(nodes: FileNode[], counts: Map<string, number>): void {
    for (const node of nodes) {
      if (node.is_dir && node.children) {
        this.countTypes(node.children, counts);
      } else if (node.asset_type) {
        counts.set(node.asset_type, (counts.get(node.asset_type) ?? 0) + 1);
      }
    }
  }

  /**
   * Dispose all sub-components.
   */
  dispose(): void {
    if (this.keydownHandler) {
      document.removeEventListener("keydown", this.keydownHandler);
    }
    this.searchBar?.dispose();
    this.filterBar?.dispose();
    this.fileTree?.dispose();
    this.batchActionBar?.dispose();
    this.quickOpen.hide();
    this.container.innerHTML = "";
  }
}
