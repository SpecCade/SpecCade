export class FilterBar {
  private container: HTMLElement;
  private onChange: (activeTypes: Set<string> | null) => void;
  private chipElements: Map<string, HTMLElement> = new Map();
  private activeTypes: Set<string> = new Set();
  private counts: Map<string, number> = new Map();
  private isAllActive: boolean = true;

  private filters = [
    { key: "all", label: "All" },
    { key: "audio", label: "🔊 Audio" },
    { key: "music", label: "🎵 Music" },
    { key: "texture", label: "🎨 Texture" },
    { key: "mesh", label: "📦 Mesh" },
    { key: "skeletal_mesh", label: "🧍 Skeletal" },
    { key: "skeletal_animation", label: "🏃 Animation" },
  ];

  constructor(
    container: HTMLElement,
    onChange: (activeTypes: Set<string> | null) => void
  ) {
    this.container = container;
    this.onChange = onChange;
    this.render();
  }

  private render(): void {
    this.container.style.cssText = `
      padding: 6px 12px;
      border-bottom: 1px solid #333;
      background: #252526;
      display: flex;
      flex-wrap: wrap;
      gap: 2px;
    `;

    for (const filter of this.filters) {
      const chip = document.createElement("div");
      chip.style.cssText = `
        padding: 3px 8px;
        border-radius: 10px;
        font-size: 11px;
        cursor: pointer;
        margin: 2px;
        user-select: none;
        white-space: nowrap;
      `;

      this.updateChipStyle(chip, filter.key);
      this.updateChipLabel(chip, filter.key, filter.label);

      chip.addEventListener("click", () => this.handleChipClick(filter.key));

      this.chipElements.set(filter.key, chip);
      this.container.appendChild(chip);
    }
  }

  private updateChipStyle(chip: HTMLElement, key: string): void {
    const isActive =
      key === "all" ? this.isAllActive : this.activeTypes.has(key);

    if (isActive) {
      chip.style.background = "#0e639c";
      chip.style.color = "#fff";
    } else {
      chip.style.background = "#333";
      chip.style.color = "#999";
    }
  }

  private updateChipLabel(
    chip: HTMLElement,
    key: string,
    baseLabel: string
  ): void {
    const count = this.counts.get(key);
    chip.textContent = count !== undefined ? `${baseLabel} (${count})` : baseLabel;
  }

  private handleChipClick(key: string): void {
    if (key === "all") {
      // Activate "All", deactivate everything else
      this.isAllActive = true;
      this.activeTypes.clear();
      this.updateAllChips();
      this.onChange(null);
    } else {
      // Deactivate "All"
      this.isAllActive = false;

      // Toggle the specific type
      if (this.activeTypes.has(key)) {
        this.activeTypes.delete(key);
      } else {
        this.activeTypes.add(key);
      }

      // If no types are active, re-activate "All"
      if (this.activeTypes.size === 0) {
        this.isAllActive = true;
        this.updateAllChips();
        this.onChange(null);
      } else {
        this.updateAllChips();
        this.onChange(new Set(this.activeTypes));
      }
    }
  }

  private updateAllChips(): void {
    for (const [key, chip] of this.chipElements) {
      this.updateChipStyle(chip, key);
    }
  }

  public setCounts(counts: Map<string, number>): void {
    this.counts = counts;

    for (const filter of this.filters) {
      const chip = this.chipElements.get(filter.key);
      if (chip) {
        this.updateChipLabel(chip, filter.key, filter.label);
      }
    }
  }

  public dispose(): void {
    for (const chip of this.chipElements.values()) {
      chip.remove();
    }
    this.chipElements.clear();
  }
}
