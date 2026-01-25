/**
 * Stdlib palette component for function discovery and insertion.
 *
 * Displays a categorized list of stdlib functions that can be inserted
 * into the editor with a single click.
 */

/**
 * Information about a stdlib function.
 */
export interface FunctionInfo {
  /** Function name */
  name: string;
  /** Function signature (e.g., "oscillator(freq, waveform, ...)" ) */
  signature: string;
  /** Brief description of what the function does */
  description: string;
  /** Code snippet to insert when clicked */
  snippet: string;
}

/**
 * Information about a category of functions.
 */
export interface CategoryInfo {
  /** Category name */
  name: string;
  /** Emoji icon for the category */
  icon: string;
  /** Functions in this category */
  functions: FunctionInfo[];
}

/**
 * Stdlib categories with their functions.
 */
export const STDLIB_CATEGORIES: CategoryInfo[] = [
  {
    name: "Audio Synthesis",
    icon: "\uD83D\uDD0A",
    functions: [
      {
        name: "oscillator",
        signature: "oscillator(freq, waveform, freq_end, sweep)",
        description: "Basic oscillator synthesis with optional frequency sweep",
        snippet: `oscillator(440, "sine", 220, "linear")`,
      },
      {
        name: "fm_synth",
        signature: "fm_synth(carrier_freq, mod_freq, mod_depth, ...)",
        description: "FM synthesis with carrier and modulator",
        snippet: `fm_synth(440, 880, 100, 50, "exponential")`,
      },
      {
        name: "envelope",
        signature: "envelope(attack, decay, sustain, release)",
        description: "ADSR amplitude envelope",
        snippet: `envelope(0.01, 0.1, 0.7, 0.2)`,
      },
      {
        name: "audio_layer",
        signature: "audio_layer(synthesis, envelope, filter, ...)",
        description: "Complete audio synthesis layer combining synthesis, envelope, and effects",
        snippet: `audio_layer(
    oscillator(440, "sawtooth"),
    envelope(0.01, 0.1, 0.5, 0.2),
    filter = lowpass(2000, 0.707)
)`,
      },
    ],
  },
  {
    name: "Audio Filters",
    icon: "\uD83C\uDFDB\uFE0F",
    functions: [
      {
        name: "lowpass",
        signature: "lowpass(cutoff, resonance, cutoff_end)",
        description: "Lowpass filter with optional cutoff sweep",
        snippet: `lowpass(2000, 0.707, 500)`,
      },
      {
        name: "highpass",
        signature: "highpass(cutoff, resonance, cutoff_end)",
        description: "Highpass filter with optional cutoff sweep",
        snippet: `highpass(200, 0.707)`,
      },
      {
        name: "reverb",
        signature: "reverb(room_size, damping, wet, dry, ...)",
        description: "Reverb effect with room simulation",
        snippet: `reverb(0.7, 0.5, 0.3, 0.7)`,
      },
    ],
  },
  {
    name: "Texture Nodes",
    icon: "\uD83C\uDFA8",
    functions: [
      {
        name: "noise_node",
        signature: "noise_node(id, noise_type, scale, octaves)",
        description: "Noise texture node (perlin, simplex, voronoi, etc.)",
        snippet: `noise_node("base", "perlin", 0.05, 6)`,
      },
      {
        name: "color_ramp_node",
        signature: "color_ramp_node(id, input, colors)",
        description: "Map grayscale values to colors via a gradient ramp",
        snippet: `color_ramp_node("colored", "base", ["#000000", "#ff6b35", "#ffffff"])`,
      },
      {
        name: "texture_graph",
        signature: "texture_graph(dimensions, nodes)",
        description: "Complete texture graph with node definitions",
        snippet: `texture_graph(
    [256, 256],
    [
        noise_node("base", "perlin", 0.05, 6),
        color_ramp_node("out", "base", ["#000000", "#ffffff"])
    ]
)`,
      },
    ],
  },
  {
    name: "Mesh",
    icon: "\uD83D\uDCE6",
    functions: [
      {
        name: "mesh_recipe",
        signature: "mesh_recipe(primitive, dimensions, modifiers)",
        description: "Complete mesh recipe with primitive and modifiers",
        snippet: `mesh_recipe(
    "cube",
    [1.0, 1.0, 1.0],
    [bevel_modifier(0.1, 3)]
)`,
      },
      {
        name: "bevel_modifier",
        signature: "bevel_modifier(width, segments)",
        description: "Bevel edges for smoother corners",
        snippet: `bevel_modifier(0.1, 3)`,
      },
      {
        name: "subdivision_modifier",
        signature: "subdivision_modifier(levels)",
        description: "Catmull-Clark subdivision surface",
        snippet: `subdivision_modifier(2)`,
      },
    ],
  },
  {
    name: "Core",
    icon: "\u2699\uFE0F",
    functions: [
      {
        name: "spec",
        signature: "spec(asset_id, asset_type, seed, outputs, recipe)",
        description: "Create a complete asset specification",
        snippet: `spec(
    asset_id = "my-asset-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("output/file.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {}
    }
)`,
      },
      {
        name: "output",
        signature: "output(path, format)",
        description: "Define an output file path and format",
        snippet: `output("output/file.wav", "wav")`,
      },
    ],
  },
];

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
  private expandedCategories: Set<string> = new Set();

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
    for (const category of STDLIB_CATEGORIES) {
      this.expandedCategories.add(category.name);
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
    this.wrapper.appendChild(header);

    // Render categories
    this.renderCategories();

    this.container.appendChild(this.wrapper);
  }

  /**
   * Render all categories.
   */
  private renderCategories(): void {
    for (const category of STDLIB_CATEGORIES) {
      const categoryElement = this.createCategory(category);
      this.wrapper.appendChild(categoryElement);
    }
  }

  /**
   * Create a collapsible category element.
   */
  private createCategory(category: CategoryInfo): HTMLDivElement {
    const categoryDiv = document.createElement("div");
    categoryDiv.style.cssText = `
      border-bottom: 1px solid #333;
    `;

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
    if (!this.expandedCategories.has(category.name)) {
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
      display: ${this.expandedCategories.has(category.name) ? "block" : "none"};
    `;

    for (const func of category.functions) {
      const funcElement = this.createFunctionItem(func);
      functionsDiv.appendChild(funcElement);
    }

    categoryDiv.appendChild(functionsDiv);

    // Toggle collapse on header click
    headerDiv.addEventListener("click", () => {
      const isExpanded = this.expandedCategories.has(category.name);
      if (isExpanded) {
        this.expandedCategories.delete(category.name);
        functionsDiv.style.display = "none";
        arrow.style.transform = "rotate(-90deg)";
      } else {
        this.expandedCategories.add(category.name);
        functionsDiv.style.display = "block";
        arrow.style.transform = "rotate(0deg)";
      }
    });

    return categoryDiv;
  }

  /**
   * Create a function item element.
   */
  private createFunctionItem(func: FunctionInfo): HTMLDivElement {
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
    for (const category of STDLIB_CATEGORIES) {
      this.expandedCategories.add(category.name);
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
