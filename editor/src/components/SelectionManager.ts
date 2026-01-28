/**
 * Multi-select state manager for file selection
 */
export class SelectionManager {
  private selected: Set<string> = new Set();
  private callbacks: SelectionCallbacks;

  constructor(callbacks: SelectionCallbacks) {
    this.callbacks = callbacks;
  }

  toggle(path: string): void {
    if (this.selected.has(path)) {
      this.deselect(path);
    } else {
      this.select(path);
    }
  }

  select(path: string): void {
    this.selected.add(path);
    this.callbacks.onSelectionChange(new Set(this.selected));
  }

  deselect(path: string): void {
    this.selected.delete(path);
    this.callbacks.onSelectionChange(new Set(this.selected));
  }

  selectRange(paths: string[]): void {
    for (const path of paths) {
      this.selected.add(path);
    }
    this.callbacks.onSelectionChange(new Set(this.selected));
  }

  selectAll(paths: string[]): void {
    for (const path of paths) {
      this.selected.add(path);
    }
    this.callbacks.onSelectionChange(new Set(this.selected));
  }

  clear(): void {
    this.selected.clear();
    this.callbacks.onSelectionChange(new Set(this.selected));
  }

  isSelected(path: string): boolean {
    return this.selected.has(path);
  }

  getSelected(): string[] {
    return Array.from(this.selected);
  }

  count(): number {
    return this.selected.size;
  }

  onValidate(): void {
    const paths = this.getSelected();
    if (paths.length > 0) {
      this.callbacks.onValidate(paths);
    }
  }

  onGenerate(): void {
    const paths = this.getSelected();
    if (paths.length > 0) {
      this.callbacks.onGenerate(paths);
    }
  }

  onDelete(): void {
    const paths = this.getSelected();
    if (paths.length > 0) {
      this.callbacks.onDelete(paths);
    }
  }
}

export interface SelectionCallbacks {
  onSelectionChange: (selected: Set<string>) => void;
  onValidate: (paths: string[]) => void;
  onGenerate: (paths: string[]) => void;
  onDelete: (paths: string[]) => void;
}

/**
 * Visual batch action bar component
 * Appears at bottom of sidebar when files are selected
 */
export class BatchActionBar {
  private container: HTMLElement;
  private selectionManager: SelectionManager;
  private barElement: HTMLElement | null = null;

  constructor(container: HTMLElement, selectionManager: SelectionManager) {
    this.container = container;
    this.selectionManager = selectionManager;
    this.render();
  }

  private render(): void {
    // Create bar container
    this.barElement = document.createElement('div');
    this.barElement.style.cssText = `
      background: #252526;
      border-top: 1px solid #333;
      padding: 8px 12px;
      position: sticky;
      bottom: 0;
      display: none;
    `;

    // Top row: count and clear button
    const topRow = document.createElement('div');
    topRow.style.cssText = `
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 6px;
    `;

    const countLabel = document.createElement('span');
    countLabel.className = 'selection-count';
    countLabel.style.cssText = `
      font-size: 12px;
      color: #ccc;
    `;

    const clearButton = document.createElement('button');
    clearButton.textContent = '×';
    clearButton.style.cssText = `
      background: none;
      border: none;
      color: #999;
      cursor: pointer;
      font-size: 18px;
      padding: 0;
      width: 20px;
      height: 20px;
      display: flex;
      align-items: center;
      justify-content: center;
    `;
    clearButton.addEventListener('click', () => {
      this.selectionManager.clear();
      this.update();
    });
    clearButton.addEventListener('mouseenter', () => {
      clearButton.style.color = '#fff';
    });
    clearButton.addEventListener('mouseleave', () => {
      clearButton.style.color = '#999';
    });

    topRow.appendChild(countLabel);
    topRow.appendChild(clearButton);

    // Bottom row: action buttons
    const buttonRow = document.createElement('div');
    buttonRow.style.cssText = `
      display: flex;
      gap: 6px;
    `;

    // Validate button
    const validateButton = this.createActionButton('Validate', '#0e639c', () => {
      this.selectionManager.onValidate();
    });

    // Generate button
    const generateButton = this.createActionButton('Generate', '#0e639c', () => {
      this.selectionManager.onGenerate();
    });

    // Delete button
    const deleteButton = this.createActionButton('Delete', '#a1260d', () => {
      this.selectionManager.onDelete();
    });

    buttonRow.appendChild(validateButton);
    buttonRow.appendChild(generateButton);
    buttonRow.appendChild(deleteButton);

    this.barElement.appendChild(topRow);
    this.barElement.appendChild(buttonRow);
    this.container.appendChild(this.barElement);
  }

  private createActionButton(
    label: string,
    backgroundColor: string,
    onClick: () => void
  ): HTMLButtonElement {
    const button = document.createElement('button');
    button.textContent = label;
    button.style.cssText = `
      background: ${backgroundColor};
      color: white;
      padding: 4px 8px;
      border-radius: 3px;
      font-size: 11px;
      border: none;
      cursor: pointer;
      flex: 1;
    `;

    button.addEventListener('click', onClick);
    button.addEventListener('mouseenter', () => {
      button.style.opacity = '0.9';
    });
    button.addEventListener('mouseleave', () => {
      button.style.opacity = '1';
    });

    return button;
  }

  update(): void {
    if (!this.barElement) return;

    const count = this.selectionManager.count();

    // Update count label
    const countLabel = this.barElement.querySelector('.selection-count');
    if (countLabel) {
      countLabel.textContent = `${count} selected`;
    }

    // Show/hide bar based on selection count
    if (count > 0) {
      this.barElement.style.display = 'block';
    } else {
      this.barElement.style.display = 'none';
    }
  }

  dispose(): void {
    if (this.barElement && this.barElement.parentNode) {
      this.barElement.parentNode.removeChild(this.barElement);
    }
    this.barElement = null;
  }
}
