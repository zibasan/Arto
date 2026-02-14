import "../style/main.css";

import { type Theme, getSystemTheme } from "./theme";
import * as markdownViewer from "./markdown-viewer";
import * as syntaxHighlighter from "./syntax-highlighter";
import * as mermaidRenderer from "./mermaid-renderer";
import { renderCoordinator } from "./render-coordinator";
import {
  setup as setupContextMenu,
  restoreSelection,
  cleanupElementReferences,
  getSavedMermaidElement,
  getSavedMathElement,
} from "./context-menu-handler";
import { rasterizeMathBlock, rasterizeMermaidBlock } from "./special-block-rasterizer";
import * as findInPage from "./find-in-page";

// Declare global Arto namespace
declare global {
  interface Window {
    Arto: {
      setupContextMenu: typeof setupContextMenu;
      restoreSelection: typeof restoreSelection;
      /** Register a callback to be called when rendering (Mermaid, KaTeX, etc.) completes */
      onRenderComplete: (callback: () => void) => void;
      /** Force a render pass for any content already in the DOM */
      scheduleRender: () => void;
      /** Rasterize an image to a PNG data URL via Canvas.
       *  SVG images are rendered at 2x scale for Retina quality.
       *  Raster images use 1x scale to preserve original resolution. */
      rasterizeImage: (src: string, opaque: boolean) => Promise<string | null>;
      /** Rasterize a Math block (KaTeX) to PNG data URL via html2canvas. */
      rasterizeMathBlock: (opaque: boolean) => Promise<string | null>;
      /** Rasterize a Mermaid SVG to PNG data URL. */
      rasterizeMermaidBlock: (opaque: boolean) => Promise<string | null>;
      /** Cleanup saved element references when context menu closes. */
      cleanupElementReferences: () => void;
      search: {
        setup: typeof findInPage.setup;
        find: typeof findInPage.find;
        navigate: typeof findInPage.navigate;
        navigateTo: typeof findInPage.navigateTo;
        clear: typeof findInPage.clear;
        reapply: typeof findInPage.reapply;
        setPinned: typeof findInPage.setPinned;
        scrollToPinnedMatch: typeof findInPage.scrollToPinnedMatch;
      };
    };
  }
}

function getCurrentTheme(): Theme {
  const theme = document.body.getAttribute("data-theme");
  switch (theme) {
    case "light":
    case "dark":
      return theme;
    default:
      return getSystemTheme();
  }
}

export function setCurrentTheme(theme: Theme) {
  document.body.setAttribute("data-theme", theme);
  markdownViewer.setTheme(theme);
  syntaxHighlighter.setTheme(theme);
  mermaidRenderer.setTheme(theme);
  renderCoordinator.forceRenderMermaid();
}

export function init(): void {
  markdownViewer.mount();
  syntaxHighlighter.mount();
  mermaidRenderer.init();
  renderCoordinator.init();

  // Expose Arto API on window for Rust interop
  window.Arto = {
    setupContextMenu,
    restoreSelection,
    onRenderComplete: (callback) => renderCoordinator.onRenderComplete(callback),
    scheduleRender: () => renderCoordinator.scheduleRender(),
    rasterizeImage: (src: string, opaque: boolean): Promise<string | null> => {
      return new Promise((resolve) => {
        const img = new Image();
        img.onload = () => {
          try {
            // SVG: 2x for Retina (vector scales perfectly)
            // Raster: 1x to preserve original resolution
            const isSvg = src.startsWith("data:image/svg") || src.endsWith(".svg");
            const scale = isSvg ? 2 : 1;
            const maxDimension = 16384;
            // ~256 MP limit prevents excessive memory allocation
            // (each pixel = 4 bytes RGBA, so 256M * 4 = ~1 GB max)
            const maxPixels = 256_000_000;
            const scaledWidth = img.naturalWidth * scale;
            const scaledHeight = img.naturalHeight * scale;
            if (
              scaledWidth > maxDimension ||
              scaledHeight > maxDimension ||
              scaledWidth * scaledHeight > maxPixels
            ) {
              console.error(`Image too large to rasterize: ${scaledWidth}x${scaledHeight}`);
              resolve(null);
              return;
            }
            const canvas = document.createElement("canvas");
            canvas.width = scaledWidth;
            canvas.height = scaledHeight;
            const ctx = canvas.getContext("2d");
            if (!ctx) {
              console.error("Failed to get 2D canvas context");
              resolve(null);
              return;
            }
            ctx.scale(scale, scale);
            if (opaque) {
              const bgColor =
                getComputedStyle(document.body).getPropertyValue("--bg-color").trim() || "#ffffff";
              ctx.fillStyle = bgColor;
              ctx.fillRect(0, 0, img.naturalWidth, img.naturalHeight);
            }
            ctx.drawImage(img, 0, 0);
            resolve(canvas.toDataURL("image/png"));
          } catch (e) {
            console.error("Failed to rasterize image:", e);
            resolve(null);
          }
        };
        img.onerror = () => resolve(null);
        img.src = src;
      });
    },
    rasterizeMathBlock: async (opaque: boolean): Promise<string | null> => {
      const element = getSavedMathElement();
      if (!element) {
        console.error("No saved Math element found");
        return null;
      }
      return rasterizeMathBlock(element, opaque);
    },
    rasterizeMermaidBlock: async (opaque: boolean): Promise<string | null> => {
      const element = getSavedMermaidElement();
      if (!element) {
        console.error("No saved Mermaid element found");
        return null;
      }
      return rasterizeMermaidBlock(element, opaque);
    },
    cleanupElementReferences,
    search: {
      setup: findInPage.setup,
      find: findInPage.find,
      navigate: findInPage.navigate,
      navigateTo: findInPage.navigateTo,
      clear: findInPage.clear,
      reapply: findInPage.reapply,
      setPinned: findInPage.setPinned,
      scrollToPinnedMatch: findInPage.scrollToPinnedMatch,
    },
  };

  // Listen for theme changes from Rust
  document.addEventListener("arto:theme-changed", ((event: CustomEvent) => {
    setCurrentTheme(event.detail);
  }) as EventListener);

  // Set initial theme
  setCurrentTheme(getCurrentTheme());
}

// Re-export mermaid window functions
export { initMermaidWindow } from "./mermaid-window-controller";
