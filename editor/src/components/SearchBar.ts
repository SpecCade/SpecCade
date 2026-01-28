/**
 * Fuzzy match utility for file search.
 * Returns match result with score and character indices for highlighting.
 */
export function fuzzyMatch(
  query: string,
  target: string
): { matches: boolean; score: number; indices: number[] } {
  if (!query) {
    return { matches: true, score: 0, indices: [] };
  }

  const queryLower = query.toLowerCase();
  const targetLower = target.toLowerCase();
  const indices: number[] = [];

  let queryIndex = 0;
  let targetIndex = 0;
  let consecutiveMatches = 0;
  let score = 0;

  while (queryIndex < queryLower.length && targetIndex < targetLower.length) {
    if (queryLower[queryIndex] === targetLower[targetIndex]) {
      indices.push(targetIndex);

      // Bonus for consecutive matches
      if (queryIndex > 0 && indices[queryIndex - 1] === targetIndex - 1) {
        consecutiveMatches++;
        score += 5 * consecutiveMatches;
      } else {
        consecutiveMatches = 0;
      }

      // Bonus for word boundary matches (after /, _, -, or start)
      if (
        targetIndex === 0 ||
        targetLower[targetIndex - 1] === "/" ||
        targetLower[targetIndex - 1] === "_" ||
        targetLower[targetIndex - 1] === "-" ||
        targetLower[targetIndex - 1] === " "
      ) {
        score += 10;
      }

      // Base score for match
      score += 1;

      queryIndex++;
    }
    targetIndex++;
  }

  const matches = queryIndex === queryLower.length;

  if (matches) {
    // Bonus for shorter targets (prefer exact matches)
    score += (1000 - target.length);

    // Bonus for matches closer to the start
    if (indices.length > 0) {
      score += (100 - indices[0]);
    }
  } else {
    score = 0;
  }

  return { matches, score, indices };
}

/**
 * Fuzzy search bar component for filtering file lists.
 */
export class SearchBar {
  private container: HTMLElement;
  private wrapper: HTMLElement;
  private inputContainer: HTMLElement;
  private input: HTMLInputElement;
  private clearButton: HTMLElement;
  private onChange: (query: string) => void;
  private debounceTimer: number | null = null;

  constructor(container: HTMLElement, onChange: (query: string) => void) {
    this.container = container;
    this.onChange = onChange;

    // Create wrapper
    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      padding: 8px 12px;
      background: #252526;
      border-bottom: 1px solid #333;
      position: relative;
    `;

    // Create input container for positioning
    this.inputContainer = document.createElement("div");
    this.inputContainer.style.cssText = `
      position: relative;
      display: flex;
      align-items: center;
    `;

    // Create input
    this.input = document.createElement("input");
    this.input.type = "text";
    this.input.placeholder = "Search files...";
    this.input.style.cssText = `
      width: 100%;
      background: #3c3c3c;
      border: 1px solid #333;
      color: #ccc;
      padding: 6px 28px 6px 8px;
      border-radius: 3px;
      font-size: 12px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      outline: none;
      box-sizing: border-box;
    `;

    // Create clear button
    this.clearButton = document.createElement("span");
    this.clearButton.textContent = "×";
    this.clearButton.style.cssText = `
      position: absolute;
      right: 8px;
      color: #999;
      cursor: pointer;
      font-size: 14px;
      user-select: none;
      display: none;
      line-height: 1;
    `;

    // Event listeners
    this.input.addEventListener("input", this.handleInput);
    this.input.addEventListener("focus", this.handleFocus);
    this.input.addEventListener("blur", this.handleBlur);
    this.input.addEventListener("keydown", this.handleKeyDown);
    this.clearButton.addEventListener("click", this.handleClear);
    this.clearButton.addEventListener("mouseenter", this.handleClearHover);
    this.clearButton.addEventListener("mouseleave", this.handleClearUnhover);

    // Assemble DOM
    this.inputContainer.appendChild(this.input);
    this.inputContainer.appendChild(this.clearButton);
    this.wrapper.appendChild(this.inputContainer);
    this.container.appendChild(this.wrapper);
  }

  private handleInput = (): void => {
    const query = this.input.value;

    // Show/hide clear button
    this.clearButton.style.display = query ? "block" : "none";

    // Debounce the onChange callback
    if (this.debounceTimer !== null) {
      clearTimeout(this.debounceTimer);
    }

    this.debounceTimer = window.setTimeout(() => {
      this.onChange(query);
      this.debounceTimer = null;
    }, 150);
  };

  private handleFocus = (): void => {
    this.input.style.borderColor = "#007acc";
  };

  private handleBlur = (): void => {
    this.input.style.borderColor = "#333";
  };

  private handleKeyDown = (e: KeyboardEvent): void => {
    if (e.key === "Escape") {
      e.preventDefault();
      this.clear();
    }
  };

  private handleClear = (): void => {
    this.clear();
  };

  private handleClearHover = (): void => {
    this.clearButton.style.color = "#ccc";
  };

  private handleClearUnhover = (): void => {
    this.clearButton.style.color = "#999";
  };

  /**
   * Clear the search input and trigger onChange with empty string.
   */
  public clear(): void {
    this.input.value = "";
    this.clearButton.style.display = "none";

    // Clear debounce timer
    if (this.debounceTimer !== null) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }

    this.onChange("");
  }

  /**
   * Focus the search input.
   */
  public focus(): void {
    this.input.focus();
  }

  /**
   * Get the current search query.
   */
  public getValue(): string {
    return this.input.value;
  }

  /**
   * Clean up event listeners and timers.
   */
  public dispose(): void {
    if (this.debounceTimer !== null) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }

    this.input.removeEventListener("input", this.handleInput);
    this.input.removeEventListener("focus", this.handleFocus);
    this.input.removeEventListener("blur", this.handleBlur);
    this.input.removeEventListener("keydown", this.handleKeyDown);
    this.clearButton.removeEventListener("click", this.handleClear);
    this.clearButton.removeEventListener("mouseenter", this.handleClearHover);
    this.clearButton.removeEventListener("mouseleave", this.handleClearUnhover);

    if (this.wrapper.parentNode) {
      this.wrapper.parentNode.removeChild(this.wrapper);
    }
  }
}
