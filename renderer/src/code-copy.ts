import iconCopy from "@tabler/icons/outline/copy.svg?raw";
import iconCheck from "@tabler/icons/outline/check.svg?raw";
import iconX from "@tabler/icons/outline/x.svg?raw";
import iconPhoto from "@tabler/icons/outline/photo.svg?raw";

/**
 * Add copy buttons to code blocks
 */
export function addCopyButtons(container: Element): void {
  const preElements = container.querySelectorAll("pre:not([data-copy-button-added])");

  if (preElements.length === 0) {
    return;
  }

  preElements.forEach((pre) => {
    addCopyButton(pre as HTMLPreElement);
  });
}

function addCopyButton(pre: HTMLPreElement): void {
  // Mark as processed
  pre.dataset.copyButtonAdded = "yes";

  // Make pre element relative for absolute positioning of button
  pre.style.position = "relative";

  // Check if this is a Mermaid diagram
  const isMermaid = pre.classList.contains("preprocessed-mermaid");

  // Create text copy button
  const textButton = document.createElement("button");
  textButton.className = isMermaid ? "copy-button copy-button-text" : "copy-button";
  textButton.setAttribute("aria-label", "Copy code to clipboard");
  textButton.innerHTML = getCopyIcon();

  // Handle click event
  textButton.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    await copyToClipboard(pre, textButton);
  });

  // Add button to pre element
  pre.appendChild(textButton);

  // Add image copy button for Mermaid
  if (isMermaid) {
    addImageCopyButton(pre);
  }
}

function addImageCopyButton(pre: HTMLPreElement): void {
  const button = document.createElement("button");
  button.className = "copy-button copy-button-image";
  button.setAttribute("aria-label", "Copy diagram as image");
  button.innerHTML = getPhotoIcon();

  button.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    await copyMermaidAsImage(pre, button);
  });

  pre.appendChild(button);
}

async function copyToClipboard(pre: HTMLPreElement, button: HTMLButtonElement): Promise<void> {
  try {
    const content = getContentToCopy(pre);
    await navigator.clipboard.writeText(content);
    showSuccessFeedback(button);
  } catch (error) {
    console.error("Failed to copy text to clipboard", error);
    showErrorFeedback(button);
  }
}

function getContentToCopy(pre: HTMLPreElement): string {
  // Check if data-original-content exists (for math and mermaid)
  const originalContent = pre.dataset.originalContent;
  if (originalContent) {
    return originalContent;
  }

  // Otherwise, get text content from code element or pre itself
  const codeElement = pre.querySelector("code");
  if (codeElement) {
    return codeElement.textContent || "";
  }

  return pre.textContent || "";
}

function showSuccessFeedback(button: HTMLButtonElement): void {
  button.innerHTML = getCheckIcon();
  button.classList.add("copied");

  // Reset after 2 seconds
  setTimeout(() => {
    button.innerHTML = getCopyIcon();
    button.classList.remove("copied");
  }, 2000);
}

function showErrorFeedback(button: HTMLButtonElement): void {
  button.innerHTML = getErrorIcon();
  button.classList.add("error");

  // Reset after 2 seconds
  setTimeout(() => {
    button.innerHTML = getCopyIcon();
    button.classList.remove("error");
  }, 2000);
}

// SVG Icons from @tabler/icons
function getCopyIcon(): string {
  return iconCopy;
}

function getCheckIcon(): string {
  return iconCheck;
}

function getErrorIcon(): string {
  return iconX;
}

function getPhotoIcon(): string {
  return iconPhoto;
}

async function copyMermaidAsImage(pre: HTMLPreElement, button: HTMLButtonElement): Promise<void> {
  if (!navigator.clipboard?.write) {
    showErrorFeedback(button);
    return;
  }

  try {
    const svg = findSvgElement(pre);
    const dimensions = getSvgDimensions(svg);
    const canvas = createCanvasFromSvg(svg, dimensions);
    const svgDataUrl = convertSvgToDataUrl(svg, dimensions);

    // Create blob promise synchronously to preserve user gesture context
    const blobPromise = createBlobPromise(canvas, svgDataUrl);

    // Write to clipboard with promise (WebKit-compatible approach)
    await navigator.clipboard.write([new ClipboardItem({ "image/png": blobPromise })]);

    showSuccessFeedback(button);
  } catch (error) {
    console.error("Failed to copy image to clipboard", error);
    showErrorFeedback(button);
  }
}

/** Find SVG element in a container */
export function findSvgElement(container: Element): SVGElement {
  const svg = container.querySelector("svg");
  if (!svg) {
    throw new Error("No SVG element found");
  }
  return svg;
}

/** Get SVG dimensions, preferring viewBox over getBBox.
 *  Mermaid sets viewBox to the full intended rendering area during diagram
 *  generation, whereas getBBox() returns only the tight content bounds which
 *  may be smaller (especially for Gantt charts). */
export function getSvgDimensions(svg: SVGElement): { width: number; height: number } {
  // Prefer viewBox dimensions (matches mermaid-window-controller approach)
  const viewBox = svg.getAttribute("viewBox");
  if (viewBox) {
    const parts = viewBox.split(/[\s,]+/).map(Number);
    if (parts.length === 4 && parts[2] > 0 && parts[3] > 0) {
      return { width: parts[2], height: parts[3] };
    }
  }

  // Fallback to getBBox for SVGs without viewBox
  const bbox = svg.getBBox();
  if (bbox.width === 0 || bbox.height === 0) {
    throw new Error("Invalid SVG dimensions");
  }

  return { width: bbox.width, height: bbox.height };
}

/** Create a canvas with the SVG background color applied */
export function createCanvasFromSvg(
  svg: SVGElement,
  dimensions: { width: number; height: number },
): HTMLCanvasElement {
  const scale = 2; // High resolution
  const canvas = document.createElement("canvas");
  canvas.width = dimensions.width * scale;
  canvas.height = dimensions.height * scale;

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Failed to get canvas context");
  }

  ctx.scale(scale, scale);

  // Get background color from current theme (use body where data-theme is set)
  const bgColor = getComputedStyle(document.body).getPropertyValue("--bg-color").trim();
  ctx.fillStyle = bgColor || "#ffffff";
  ctx.fillRect(0, 0, dimensions.width, dimensions.height);

  return canvas;
}

/** Convert SVG element to data URL.
 *  Resolves inherited styles (e.g., font-family) so the SVG renders
 *  correctly when loaded as a standalone image. */
export function convertSvgToDataUrl(
  svg: SVGElement,
  dimensions: { width: number; height: number },
): string {
  const svgClone = svg.cloneNode(true) as SVGElement;
  svgClone.setAttribute("width", String(dimensions.width));
  svgClone.setAttribute("height", String(dimensions.height));

  // Resolve inherited font-family from the live DOM.
  // Mermaid uses fontFamily: "inherit" which works in-page but fails
  // when the SVG is loaded as a standalone <img> (no parent to inherit from).
  const computedFont = getComputedStyle(svg).fontFamily;
  if (computedFont) {
    svgClone.style.fontFamily = computedFont;
  }

  const svgString = new XMLSerializer().serializeToString(svgClone);
  const base64SVG = btoa(unescape(encodeURIComponent(svgString)));

  return `data:image/svg+xml;base64,${base64SVG}`;
}

/** Create a blob promise from canvas and SVG data URL */
export function createBlobPromise(canvas: HTMLCanvasElement, dataUrl: string): Promise<Blob> {
  return new Promise<Blob>((resolve, reject) => {
    const img = new Image();

    img.onload = () => {
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        reject(new Error("Canvas context lost"));
        return;
      }

      try {
        ctx.drawImage(img, 0, 0);

        canvas.toBlob((blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error("Failed to create blob"));
          }
        }, "image/png");
      } catch (error) {
        reject(error);
      }
    };

    img.onerror = () => reject(new Error("Failed to load image"));
    img.src = dataUrl;
  });
}

/** Convert a Blob to a data URL string */
function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error("Failed to read blob as data URL"));
    reader.readAsDataURL(blob);
  });
}
