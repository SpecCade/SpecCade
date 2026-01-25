/**
 * New Asset Dialog component.
 *
 * Modal dialog for creating new asset specifications from templates.
 * Displays asset type selector and loads templates via Tauri IPC.
 */
import { invoke } from "@tauri-apps/api/core";
import { AssetTypeSelector } from "./AssetTypeSelector";

/**
 * Modal dialog for creating new assets from templates.
 */
export class NewAssetDialog {
  private overlay: HTMLDivElement | null = null;
  private dialog: HTMLDivElement | null = null;
  private selector: AssetTypeSelector | null = null;
  private onTemplateLoaded: (content: string) => void;

  /**
   * Create a new asset dialog.
   *
   * @param onTemplateLoaded - Callback invoked when a template is loaded
   */
  constructor(onTemplateLoaded: (content: string) => void) {
    this.onTemplateLoaded = onTemplateLoaded;
  }

  /**
   * Load a template by ID from the backend.
   *
   * @param templateId - The template identifier to load
   */
  async loadTemplate(templateId: string): Promise<void> {
    try {
      const content = await invoke<string>("plugin:speccade|get_template", {
        id: templateId,
      });
      this.onTemplateLoaded(content);
      this.close();
    } catch (error) {
      console.error("Failed to load template:", error);
      // Show error in dialog
      if (this.dialog) {
        const errorDiv = document.createElement("div");
        errorDiv.style.cssText = `
          padding: 12px;
          background: #5a1d1d;
          color: #ff6b6b;
          border-radius: 4px;
          margin: 16px;
          font-size: 12px;
        `;
        errorDiv.textContent = `Failed to load template: ${String(error)}`;
        this.dialog.appendChild(errorDiv);
      }
    }
  }

  /**
   * Show the dialog.
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
      width: 500px;
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
    title.textContent = "Create New Asset";
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

    // Create content area
    const content = document.createElement("div");
    content.style.cssText = `
      flex: 1;
      overflow-y: auto;
    `;

    // Create asset type selector
    this.selector = new AssetTypeSelector(content, (templateId) => {
      this.loadTemplate(templateId);
    });

    this.dialog.appendChild(content);
    this.overlay.appendChild(this.dialog);
    document.body.appendChild(this.overlay);

    // Handle escape key
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        this.close();
        document.removeEventListener("keydown", handleKeyDown);
      }
    };
    document.addEventListener("keydown", handleKeyDown);
  }

  /**
   * Close and remove the dialog.
   */
  close(): void {
    if (this.selector) {
      this.selector.dispose();
      this.selector = null;
    }
    if (this.overlay) {
      document.body.removeChild(this.overlay);
      this.overlay = null;
      this.dialog = null;
    }
  }
}
