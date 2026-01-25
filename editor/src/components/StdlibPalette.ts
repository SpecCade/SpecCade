/**
 * Stdlib palette component for function discovery and insertion.
 *
 * Displays a categorized list of stdlib functions that can be inserted
 * into the editor with a single click.
 */

import {
  STDLIB_MANIFEST,
  type StdlibCategory,
  type StdlibCategoryId,
  type StdlibFunction,
} from "../lib/stdlib-manifest";

/**
 * Stdlib palette component.
 *
 * Displays collapsible categories of stdlib functions that can be
 * clicked to insert code snippets into the editor.
 */
export class StdlibPalette {
  private container: HTMLElement;
  private onInsert: (snippet: string) => void;
  private wrapper: HTMLDivElement;
  private expandedCategories: Set<StdlibCategoryId> = new Set();
  private query = "";

  /**
   * Create a new stdlib palette.
   *
   * @param container - The HTML element to render into
   * @param onInsert - Callback invoked when a function is clicked (receives snippet)
   */
  constructor(container: HTMLElement, onInsert: (snippet: string) => void) {
    this.container = container;
    this.onInsert = onInsert;

    // Start with all categories expanded
    for (const category of STDLIB_MANIFEST) {
      this.expandedCategories.add(category.id);
    }

    // Create wrapper
    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      overflow-y: auto;
      background: #1e1e1e;
    `;

    // Create header
    const header = document.createElement("div");
    header.style.cssText = `
      display: flex;
      align-items: center;
      padding: 8px 12px;
      background: #252526;
      border-bottom: 1px solid #333;
    `;

    const title = document.createElement("span");
    title.textContent = "Stdlib Functions";
    title.style.cssText = `
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      color: #999;
      letter-spacing: 0.5px;
    `;
    header.appendChild(title);

    const searchInput = document.createElement("input");
    searchInput.type = "search";
    searchInput.placeholder = "Search";
    searchInput.setAttribute("aria-label", "Search stdlib functions");
    searchInput.autocomplete = "off";
    searchInput.spellcheck = false;
    searchInput.style.cssText = `
      margin-left: auto;
      width: 160px;
      padding: 4px 6px;
      font-size: 11px;
      color: #ccc;
      background: #1e1e1e;
      border: 1px solid #333;
      border-radius: 3px;
      outline: none;
    `;
    searchInput.addEventListener("focus", () => {
      searchInput.style.borderColor = "#007acc";
    });
    searchInput.addEventListener("blur", () => {
      searchInput.style.borderColor = "#333";
    });
    searchInput.addEventListener("input", () => {
      this.query = searchInput.value.trim().toLowerCase();
      this.refresh();
    });
    header.appendChild(searchInput);
    this.wrapper.appendChild(header);

    // Render categories
    this.renderCategories();

    this.container.appendChild(this.wrapper);
  }

  /**
   * Render all categories.
   */
  private renderCategories(): void {
    for (const category of this.getVisibleCategories()) {
      const categoryElement = this.createCategory(category);
      this.wrapper.appendChild(categoryElement);
    }
  }

  private getVisibleCategories(): StdlibCategory[] {
    const query = this.query;
    if (query.length === 0) {
      return STDLIB_MANIFEST;
    }

    const out: StdlibCategory[] = [];
    for (const category of STDLIB_MANIFEST) {
      const functions = category.functions.filter((func) =>
        this.functionMatchesQuery(func, query),
      );
      if (functions.length === 0) {
        continue;
      }
      out.push({ ...category, functions });
    }
    return out;
  }

  private functionMatchesQuery(func: StdlibFunction, query: string): boolean {
    const haystack = `${func.name}\n${func.signature}\n${func.description}`.toLowerCase();
    return haystack.includes(query);
  }

  /**
   * Create a collapsible category element.
   */
  private createCategory(category: StdlibCategory): HTMLDivElement {
    const categoryDiv = document.createElement("div");
    categoryDiv.style.cssText = `
      border-bottom: 1px solid #333;
    `;

    const isSearching = this.query.length > 0;
    const isExpanded = isSearching
      ? true
      : this.expandedCategories.has(category.id);

    // Category header (collapsible)
    const headerDiv = document.createElement("div");
    headerDiv.style.cssText = `
      display: flex;
      align-items: center;
      padding: 8px 12px;
      cursor: pointer;
      background: #252526;
      transition: background 0.1s ease;
      user-select: none;
    `;

    // Hover effect for header
    headerDiv.addEventListener("mouseenter", () => {
      headerDiv.style.background = "#2a2d2e";
    });
    headerDiv.addEventListener("mouseleave", () => {
      headerDiv.style.background = "#252526";
    });

    // Expand/collapse arrow
    const arrow = document.createElement("span");
    arrow.style.cssText = `
      font-size: 10px;
      margin-right: 8px;
      color: #999;
      transition: transform 0.15s ease;
    `;
    arrow.textContent = "\u25BC";
    if (!isExpanded) {
      arrow.style.transform = "rotate(-90deg)";
    }
    headerDiv.appendChild(arrow);

    // Icon
    const icon = document.createElement("span");
    icon.textContent = category.icon;
    icon.style.cssText = `
      font-size: 14px;
      margin-right: 8px;
    `;
    headerDiv.appendChild(icon);

    // Category name
    const nameSpan = document.createElement("span");
    nameSpan.textContent = category.name;
    nameSpan.style.cssText = `
      font-size: 12px;
      font-weight: 600;
      color: #ccc;
    `;
    headerDiv.appendChild(nameSpan);

    // Function count badge
    const countBadge = document.createElement("span");
    countBadge.textContent = String(category.functions.length);
    countBadge.style.cssText = `
      margin-left: auto;
      font-size: 10px;
      color: #666;
      background: #333;
      padding: 2px 6px;
      border-radius: 10px;
    `;
    headerDiv.appendChild(countBadge);

    categoryDiv.appendChild(headerDiv);

    // Functions container
    const functionsDiv = document.createElement("div");
    functionsDiv.style.cssText = `
      display: ${isExpanded ? "block" : "none"};
    `;

    for (const func of category.functions) {
      const funcElement = this.createFunctionItem(func);
      functionsDiv.appendChild(funcElement);
    }

    categoryDiv.appendChild(functionsDiv);

    // Toggle collapse on header click
    headerDiv.addEventListener("click", () => {
      if (this.query.trim().length > 0) {
        return;
      }
      const isExpanded = this.expandedCategories.has(category.id);
      if (isExpanded) {
        this.expandedCategories.delete(category.id);
        functionsDiv.style.display = "none";
        arrow.style.transform = "rotate(-90deg)";
      } else {
        this.expandedCategories.add(category.id);
        functionsDiv.style.display = "block";
        arrow.style.transform = "rotate(0deg)";
      }
    });

    return categoryDiv;
  }

  /**
   * Create a function item element.
   */
  private createFunctionItem(func: StdlibFunction): HTMLDivElement {
    const itemDiv = document.createElement("div");
    itemDiv.style.cssText = `
      padding: 8px 12px 8px 32px;
      cursor: pointer;
      transition: background 0.1s ease;
      border-left: 2px solid transparent;
    `;

    // Hover effect
    itemDiv.addEventListener("mouseenter", () => {
      itemDiv.style.background = "#2a2d2e";
      itemDiv.style.borderLeftColor = "#007acc";
    });
    itemDiv.addEventListener("mouseleave", () => {
      itemDiv.style.background = "transparent";
      itemDiv.style.borderLeftColor = "transparent";
    });

    // Function name (monospace, blue)
    const nameSpan = document.createElement("div");
    nameSpan.textContent = func.name;
    nameSpan.style.cssText = `
      font-family: "Consolas", "Monaco", "Courier New", monospace;
      font-size: 12px;
      color: #569cd6;
      margin-bottom: 2px;
    `;
    itemDiv.appendChild(nameSpan);

    // Description
    const descSpan = document.createElement("div");
    descSpan.textContent = func.description;
    descSpan.style.cssText = `
      font-size: 11px;
      color: #888;
      line-height: 1.3;
    `;
    itemDiv.appendChild(descSpan);

    // Click handler - insert snippet
    itemDiv.addEventListener("click", () => {
      this.onInsert(func.snippet);
    });

    return itemDiv;
  }

  /**
   * Expand all categories.
   */
  expandAll(): void {
    for (const category of STDLIB_MANIFEST) {
      this.expandedCategories.add(category.id);
    }
    this.refresh();
  }

  /**
   * Collapse all categories.
   */
  collapseAll(): void {
    this.expandedCategories.clear();
    this.refresh();
  }

  /**
   * Refresh the palette by re-rendering.
   */
  private refresh(): void {
    // Remove all category elements (keep header)
    while (this.wrapper.children.length > 1) {
      this.wrapper.removeChild(this.wrapper.lastChild!);
    }
    this.renderCategories();
  }

  /**
   * Dispose of the palette and clear the container.
   */
  dispose(): void {
    this.container.innerHTML = "";
  }
}
