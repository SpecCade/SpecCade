/**
 * Asset type selector component for creating new specs.
 *
 * Displays a grid of asset type cards that users can select from
 * when creating a new asset specification.
 */

/**
 * Information about an asset type.
 */
export interface AssetTypeInfo {
  /** Unique identifier for the asset type */
  id: string;
  /** Display name for the asset type */
  name: string;
  /** Emoji icon representing the asset type */
  icon: string;
  /** Brief description of what this asset type is for */
  description: string;
  /** Template identifier to use when creating this asset type */
  template: string;
}

/**
 * Available asset types for spec creation.
 */
export const ASSET_TYPES: AssetTypeInfo[] = [
  {
    id: "audio",
    name: "Sound Effect",
    icon: "\uD83D\uDD0A",
    description: "Procedural sound effects and audio synthesis",
    template: "audio_basic",
  },
  {
    id: "music",
    name: "Music Track",
    icon: "\uD83C\uDFB5",
    description: "Tracker-style music composition",
    template: "music_basic",
  },
  {
    id: "texture",
    name: "Texture",
    icon: "\uD83C\uDFA8",
    description: "Procedural textures with node-based generation",
    template: "texture_basic",
  },
  {
    id: "static_mesh",
    name: "3D Mesh",
    icon: "\uD83D\uDCE6",
    description: "Static 3D geometry and models",
    template: "mesh_basic",
  },
  {
    id: "skeletal_mesh",
    name: "Character",
    icon: "\uD83E\uDDCD",
    description: "Rigged character meshes with skeletons",
    template: "character_basic",
  },
  {
    id: "skeletal_animation",
    name: "Animation",
    icon: "\uD83C\uDFC3",
    description: "Skeletal animations for characters",
    template: "animation_basic",
  },
  {
    id: "sprite",
    name: "Sprite Sheet",
    icon: "\uD83D\uDDBC\uFE0F",
    description: "2D sprite sheets and animations",
    template: "sprite_basic",
  },
  {
    id: "vfx",
    name: "VFX",
    icon: "\u2728",
    description: "Visual effects and particle systems",
    template: "vfx_basic",
  },
];

/**
 * Asset type selector component.
 *
 * Renders a 2-column grid of asset type cards that users can click
 * to select an asset type for creating a new spec.
 */
export class AssetTypeSelector {
  private container: HTMLElement;
  private onSelect: (templateId: string) => void;
  private wrapper: HTMLDivElement;

  /**
   * Create a new asset type selector.
   *
   * @param container - The HTML element to render into
   * @param onSelect - Callback invoked when an asset type is selected
   */
  constructor(container: HTMLElement, onSelect: (templateId: string) => void) {
    this.container = container;
    this.onSelect = onSelect;

    // Create wrapper
    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      padding: 16px;
      box-sizing: border-box;
      overflow-y: auto;
    `;

    // Create header
    const header = document.createElement("h2");
    header.textContent = "Create New Asset";
    header.style.cssText = `
      margin: 0 0 16px 0;
      font-size: 18px;
      font-weight: 600;
      color: #fff;
    `;
    this.wrapper.appendChild(header);

    // Create grid container
    const grid = document.createElement("div");
    grid.style.cssText = `
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;
    `;

    // Create cards for each asset type
    for (const assetType of ASSET_TYPES) {
      const card = this.createCard(assetType);
      grid.appendChild(card);
    }

    this.wrapper.appendChild(grid);
    this.container.appendChild(this.wrapper);
  }

  /**
   * Create a card element for an asset type.
   */
  private createCard(assetType: AssetTypeInfo): HTMLDivElement {
    const card = document.createElement("div");
    card.style.cssText = `
      display: flex;
      flex-direction: column;
      padding: 16px;
      background: #252526;
      border: 1px solid #3c3c3c;
      border-radius: 8px;
      cursor: pointer;
      transition: background 0.15s ease, border-color 0.15s ease;
    `;

    // Hover effect
    card.addEventListener("mouseenter", () => {
      card.style.background = "#333";
      card.style.borderColor = "#007acc";
    });

    card.addEventListener("mouseleave", () => {
      card.style.background = "#252526";
      card.style.borderColor = "#3c3c3c";
    });

    // Click handler
    card.addEventListener("click", () => {
      this.onSelect(assetType.template);
    });

    // Icon
    const icon = document.createElement("div");
    icon.textContent = assetType.icon;
    icon.style.cssText = `
      font-size: 32px;
      margin-bottom: 8px;
    `;
    card.appendChild(icon);

    // Name
    const name = document.createElement("div");
    name.textContent = assetType.name;
    name.style.cssText = `
      font-size: 14px;
      font-weight: 600;
      color: #fff;
      margin-bottom: 4px;
    `;
    card.appendChild(name);

    // Description
    const description = document.createElement("div");
    description.textContent = assetType.description;
    description.style.cssText = `
      font-size: 12px;
      color: #999;
      line-height: 1.4;
    `;
    card.appendChild(description);

    return card;
  }

  /**
   * Dispose of the selector and clear the container.
   */
  dispose(): void {
    this.container.innerHTML = "";
  }
}
