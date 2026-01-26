/**
 * Problems panel component.
 *
 * Renders a simple, clickable list of issues surfaced by the editor pipeline.
 */

export type ProblemStage = "eval" | "compile" | "preview" | "ipc" | "lint";

export type ProblemItem = {
  id: string;
  stage: ProblemStage;
  severity: "error" | "warning";
  code?: string;
  message: string;
  location?: { line: number; column: number };
};

function plural(n: number, singular: string, pluralWord: string): string {
  return n === 1 ? singular : pluralWord;
}

export class ProblemsPanel {
  private container: HTMLElement;
  private onSelect?: (problem: ProblemItem) => void;

  private wrapper: HTMLDivElement;
  private countsEl: HTMLDivElement;
  private listEl: HTMLDivElement;
  private emptyEl: HTMLDivElement;

  private problems: ProblemItem[] = [];

  constructor(container: HTMLElement, onSelect?: (problem: ProblemItem) => void) {
    this.container = container;
    this.onSelect = onSelect;

    this.wrapper = document.createElement("div");
    this.wrapper.style.cssText = `
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      padding: 12px;
      box-sizing: border-box;
      gap: 10px;
    `;

    const headerRow = document.createElement("div");
    headerRow.style.cssText = `
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
    `;

    const title = document.createElement("div");
    title.textContent = "Problems";
    title.style.cssText = `
      font-size: 12px;
      font-weight: 600;
      color: #ccc;
      letter-spacing: 0.2px;
    `;
    headerRow.appendChild(title);

    this.countsEl = document.createElement("div");
    this.countsEl.style.cssText = `
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 11px;
      color: #888;
      white-space: nowrap;
    `;
    headerRow.appendChild(this.countsEl);
    this.wrapper.appendChild(headerRow);

    const divider = document.createElement("div");
    divider.style.cssText = `
      height: 1px;
      background: #333;
      width: 100%;
    `;
    this.wrapper.appendChild(divider);

    this.listEl = document.createElement("div");
    this.listEl.style.cssText = `
      flex: 1;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      gap: 6px;
    `;
    this.wrapper.appendChild(this.listEl);

    this.emptyEl = document.createElement("div");
    this.emptyEl.textContent = "No problems";
    this.emptyEl.style.cssText = `
      padding: 12px;
      border: 1px dashed #3a3a3a;
      border-radius: 6px;
      background: #1a1a1a;
      color: #777;
      font-size: 12px;
      text-align: center;
    `;

    this.container.appendChild(this.wrapper);
    this.render();
  }

  setProblems(problems: ProblemItem[]): void {
    this.problems = problems.slice();
    this.render();
  }

  clear(): void {
    this.setProblems([]);
  }

  dispose(): void {
    this.container.innerHTML = "";
  }

  private render(): void {
    const errorCount = this.problems.filter((p) => p.severity === "error").length;
    const warningCount = this.problems.filter((p) => p.severity === "warning").length;

    this.countsEl.textContent = `${errorCount} ${plural(errorCount, "error", "errors")}, ${warningCount} ${plural(
      warningCount,
      "warning",
      "warnings"
    )}`;

    this.listEl.innerHTML = "";

    if (this.problems.length === 0) {
      this.listEl.appendChild(this.emptyEl);
      return;
    }

    for (const problem of this.problems) {
      const row = document.createElement("div");
      row.style.cssText = `
        display: flex;
        align-items: flex-start;
        gap: 8px;
        padding: 8px 10px;
        background: #222;
        border: 1px solid #333;
        border-radius: 6px;
        cursor: pointer;
      `;
      row.addEventListener("mouseenter", () => {
        row.style.background = "#262626";
        row.style.borderColor = "#3a3a3a";
      });
      row.addEventListener("mouseleave", () => {
        row.style.background = "#222";
        row.style.borderColor = "#333";
      });
      row.addEventListener("click", () => {
        this.onSelect?.(problem);
      });

      const badge = document.createElement("div");
      badge.textContent = problem.severity === "error" ? "E" : "W";
      badge.style.cssText = `
        width: 18px;
        height: 18px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 11px;
        font-weight: 700;
        color: ${problem.severity === "error" ? "#ffd6d1" : "#fff1c6"};
        background: ${problem.severity === "error" ? "#6b2a2a" : "#6a571e"};
        flex: 0 0 auto;
      `;
      row.appendChild(badge);

      const body = document.createElement("div");
      body.style.cssText = `
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 4px;
        min-width: 0;
      `;

      const top = document.createElement("div");
      top.style.cssText = `
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 10px;
      `;

      const label = document.createElement("div");
      const codePrefix = problem.code ? `${problem.code}: ` : "";
      label.textContent = `${codePrefix}${problem.message}`;
      label.style.cssText = `
        font-size: 12px;
        color: #ddd;
        line-height: 16px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      `;
      top.appendChild(label);

      const meta = document.createElement("div");
      const loc = problem.location ? `:${problem.location.line}:${problem.location.column}` : "";
      meta.textContent = `${problem.stage}${loc}`;
      meta.style.cssText = `
        font-size: 11px;
        color: #888;
        flex: 0 0 auto;
        white-space: nowrap;
      `;
      top.appendChild(meta);

      body.appendChild(top);
      row.appendChild(body);
      this.listEl.appendChild(row);
    }
  }
}
