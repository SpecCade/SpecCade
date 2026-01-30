/**
 * PanelResizer - Handles draggable splitters for resizable panels.
 *
 * Persists panel widths to localStorage.
 */

const STORAGE_KEY = "speccade:panel_widths:v1";

interface PanelWidths {
  sidebar: number;
  preview: number;
}

const DEFAULT_WIDTHS: PanelWidths = {
  sidebar: 220,
  preview: 400,
};

/**
 * Initialize panel resizing for the editor layout.
 *
 * Call this once after DOM is ready.
 */
export function initPanelResizers(): void {
  const sidebar = document.getElementById("sidebar");
  const previewPane = document.getElementById("preview-pane");
  const sidebarSplitter = document.getElementById("sidebar-splitter");
  const previewSplitter = document.getElementById("preview-splitter");

  if (!sidebar || !previewPane || !sidebarSplitter || !previewSplitter) {
    console.warn("PanelResizer: Missing required DOM elements");
    return;
  }

  // Load saved widths
  const widths = loadWidths();
  sidebar.style.width = `${widths.sidebar}px`;
  previewPane.style.width = `${widths.preview}px`;

  // Set up sidebar splitter
  setupSplitter(sidebarSplitter, sidebar, "left", (newWidth) => {
    const clamped = Math.max(80, Math.min(500, newWidth));
    sidebar.style.width = `${clamped}px`;
    saveWidths({ ...loadWidths(), sidebar: clamped });
    triggerEditorResize();
  });

  // Set up preview splitter
  setupSplitter(previewSplitter, previewPane, "right", (newWidth) => {
    const clamped = Math.max(100, Math.min(800, newWidth));
    previewPane.style.width = `${clamped}px`;
    saveWidths({ ...loadWidths(), preview: clamped });
    triggerEditorResize();
  });
}

function setupSplitter(
  splitter: HTMLElement,
  panel: HTMLElement,
  side: "left" | "right",
  onResize: (width: number) => void
): void {
  let isDragging = false;
  let startX = 0;
  let startWidth = 0;

  const onMouseDown = (e: MouseEvent) => {
    isDragging = true;
    startX = e.clientX;
    startWidth = panel.offsetWidth;
    splitter.classList.add("dragging");
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
  };

  const onMouseMove = (e: MouseEvent) => {
    if (!isDragging) return;

    const delta = e.clientX - startX;
    let newWidth: number;

    if (side === "left") {
      // For sidebar: dragging right increases width
      newWidth = startWidth + delta;
    } else {
      // For preview pane: dragging left increases width
      newWidth = startWidth - delta;
    }

    onResize(newWidth);
  };

  const onMouseUp = () => {
    if (!isDragging) return;
    isDragging = false;
    splitter.classList.remove("dragging");
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  };

  splitter.addEventListener("mousedown", onMouseDown);
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
}

function loadWidths(): PanelWidths {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        sidebar: parsed.sidebar ?? DEFAULT_WIDTHS.sidebar,
        preview: parsed.preview ?? DEFAULT_WIDTHS.preview,
      };
    }
  } catch {
    // ignore
  }
  return { ...DEFAULT_WIDTHS };
}

function saveWidths(widths: PanelWidths): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(widths));
}

/**
 * Trigger Monaco editor to recalculate its layout.
 */
function triggerEditorResize(): void {
  // Monaco editor listens to window resize events
  window.dispatchEvent(new Event("resize"));
}
