/**
 * Help Panel component.
 *
 * Modal dialog displaying keyboard shortcuts, asset types, and quick links
 * to documentation resources.
 */

/**
 * Modal dialog for help and quick reference information.
 */
export class HelpPanel {
  private overlay: HTMLDivElement | null = null;
  private dialog: HTMLDivElement | null = null;
  private keydownHandler: ((e: KeyboardEvent) => void) | null = null;

  /**
   * Create a new help panel.
   */
  constructor() {
    // No initialization needed
  }

  /**
   * Show the help panel.
   */
  show(): void {
    // Create overlay
    this.overlay = document.createElement("div");
    this.overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.7);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
    `;

    // Close on overlay click
    this.overlay.addEventListener("click", (e) => {
      if (e.target === this.overlay) {
        this.close();
      }
    });

    // Create dialog box
    this.dialog = document.createElement("div");
    this.dialog.style.cssText = `
      background: #1e1e1e;
      width: 550px;
      max-height: 80vh;
      border-radius: 8px;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    `;

    // Create header
    const header = document.createElement("div");
    header.style.cssText = `
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 16px;
      border-bottom: 1px solid #333;
    `;

    const title = document.createElement("h2");
    title.textContent = "Help & Keyboard Shortcuts";
    title.style.cssText = `
      margin: 0;
      font-size: 16px;
      font-weight: 600;
      color: #fff;
    `;

    const closeButton = document.createElement("button");
    closeButton.innerHTML = "&times;";
    closeButton.style.cssText = `
      background: none;
      border: none;
      color: #999;
      font-size: 24px;
      cursor: pointer;
      padding: 0;
      line-height: 1;
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 4px;
    `;
    closeButton.addEventListener("mouseenter", () => {
      closeButton.style.background = "#333";
      closeButton.style.color = "#fff";
    });
    closeButton.addEventListener("mouseleave", () => {
      closeButton.style.background = "none";
      closeButton.style.color = "#999";
    });
    closeButton.addEventListener("click", () => this.close());

    header.appendChild(title);
    header.appendChild(closeButton);
    this.dialog.appendChild(header);

    // Create content area with scrolling
    const content = document.createElement("div");
    content.style.cssText = `
      flex: 1;
      overflow-y: auto;
      padding: 16px;
    `;

    // Keyboard Shortcuts section
    content.appendChild(this.createSection("Keyboard Shortcuts", this.createShortcutsTable()));

    // Asset Types section
    content.appendChild(this.createSection("Asset Types", this.createAssetTypesTable()));

    // Quick Links section
    content.appendChild(this.createSection("Quick Links", this.createQuickLinks()));

    this.dialog.appendChild(content);
    this.overlay.appendChild(this.dialog);
    document.body.appendChild(this.overlay);

    // Handle escape key
    this.keydownHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        this.close();
      }
    };
    document.addEventListener("keydown", this.keydownHandler);
  }

  /**
   * Create a section with title and content.
   */
  private createSection(titleText: string, contentElement: HTMLElement): HTMLElement {
    const section = document.createElement("div");
    section.style.cssText = `
      margin-bottom: 20px;
    `;

    const sectionTitle = document.createElement("h3");
    sectionTitle.textContent = titleText;
    sectionTitle.style.cssText = `
      margin: 0 0 12px 0;
      font-size: 14px;
      font-weight: 600;
      color: #ccc;
      border-bottom: 1px solid #333;
      padding-bottom: 8px;
    `;

    section.appendChild(sectionTitle);
    section.appendChild(contentElement);

    return section;
  }

  /**
   * Create the keyboard shortcuts table.
   */
  private createShortcutsTable(): HTMLElement {
    const shortcuts = [
      { key: "Ctrl+N", description: "Create new asset from template" },
      { key: "Ctrl+S", description: "Save current file" },
      { key: "Ctrl+Space", description: "Trigger autocomplete" },
      { key: "F1", description: "Show this help panel" },
    ];

    return this.createTable(["Shortcut", "Action"], shortcuts.map(s => [s.key, s.description]));
  }

  /**
   * Create the asset types table.
   */
  private createAssetTypesTable(): HTMLElement {
    const assetTypes = [
      { type: "audio", description: "Sound effects and audio clips" },
      { type: "music", description: "Tracker-based music compositions" },
      { type: "texture", description: "2D textures and images" },
      { type: "static_mesh", description: "Static 3D geometry" },
      { type: "skeletal_mesh", description: "Rigged 3D meshes with skeleton" },
      { type: "skeletal_animation", description: "Animation clips for skeletal meshes" },
    ];

    return this.createTable(["Type", "Description"], assetTypes.map(a => [a.type, a.description]));
  }

  /**
   * Create a styled table element.
   */
  private createTable(headers: string[], rows: string[][]): HTMLElement {
    const table = document.createElement("table");
    table.style.cssText = `
      width: 100%;
      border-collapse: collapse;
      font-size: 12px;
    `;

    // Header row
    const thead = document.createElement("thead");
    const headerRow = document.createElement("tr");
    headers.forEach((headerText) => {
      const th = document.createElement("th");
      th.textContent = headerText;
      th.style.cssText = `
        text-align: left;
        padding: 8px;
        color: #888;
        font-weight: 500;
        border-bottom: 1px solid #333;
      `;
      headerRow.appendChild(th);
    });
    thead.appendChild(headerRow);
    table.appendChild(thead);

    // Body rows
    const tbody = document.createElement("tbody");
    rows.forEach((row) => {
      const tr = document.createElement("tr");
      row.forEach((cellText, index) => {
        const td = document.createElement("td");
        td.textContent = cellText;
        td.style.cssText = `
          padding: 8px;
          color: ${index === 0 ? "#9cdcfe" : "#ccc"};
          font-family: ${index === 0 ? "'Consolas', 'Monaco', monospace" : "inherit"};
          border-bottom: 1px solid #2a2a2a;
        `;
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);

    return table;
  }

  /**
   * Create the quick links section.
   */
  private createQuickLinks(): HTMLElement {
    const container = document.createElement("div");
    container.style.cssText = `
      display: flex;
      flex-direction: column;
      gap: 8px;
    `;

    const links = [
      {
        text: "Stdlib Reference",
        url: "https://github.com/user/speccade/blob/main/docs/stdlib-reference.md",
        description: "Complete reference for all stdlib functions",
      },
      {
        text: "Starlark Authoring Guide",
        url: "https://github.com/user/speccade/blob/main/docs/starlark-authoring.md",
        description: "Guide to writing Starlark specs",
      },
    ];

    links.forEach((link) => {
      const linkContainer = document.createElement("div");
      linkContainer.style.cssText = `
        display: flex;
        flex-direction: column;
        padding: 10px 12px;
        background: #252525;
        border-radius: 4px;
        cursor: pointer;
      `;

      linkContainer.addEventListener("mouseenter", () => {
        linkContainer.style.background = "#2a2a2a";
      });
      linkContainer.addEventListener("mouseleave", () => {
        linkContainer.style.background = "#252525";
      });

      const anchor = document.createElement("a");
      anchor.href = link.url;
      anchor.target = "_blank";
      anchor.rel = "noopener noreferrer";
      anchor.textContent = link.text;
      anchor.style.cssText = `
        color: #4fc3f7;
        text-decoration: none;
        font-size: 13px;
        font-weight: 500;
      `;
      anchor.addEventListener("mouseenter", () => {
        anchor.style.textDecoration = "underline";
      });
      anchor.addEventListener("mouseleave", () => {
        anchor.style.textDecoration = "none";
      });

      const description = document.createElement("span");
      description.textContent = link.description;
      description.style.cssText = `
        color: #888;
        font-size: 11px;
        margin-top: 4px;
      `;

      linkContainer.appendChild(anchor);
      linkContainer.appendChild(description);

      // Make the whole container clickable
      linkContainer.addEventListener("click", (e) => {
        if (e.target !== anchor) {
          window.open(link.url, "_blank", "noopener,noreferrer");
        }
      });

      container.appendChild(linkContainer);
    });

    return container;
  }

  /**
   * Close and remove the help panel.
   */
  close(): void {
    if (this.keydownHandler) {
      document.removeEventListener("keydown", this.keydownHandler);
      this.keydownHandler = null;
    }
    if (this.overlay) {
      document.body.removeChild(this.overlay);
      this.overlay = null;
      this.dialog = null;
    }
  }
}
