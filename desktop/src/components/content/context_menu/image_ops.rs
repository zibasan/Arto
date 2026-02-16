use dioxus::prelude::*;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu};

/// Copy an image to the clipboard via browser Canvas rasterization.
///
/// All image types (SVG, PNG, JPG, etc.) go through the browser's Canvas API.
/// This ensures consistent rendering (especially for SVGs with `<foreignObject>`)
/// and enables the `opaque` background option for all image types.
///
/// When `opaque` is true, the Canvas fills the background with the
/// current theme color before drawing.
pub(super) fn copy_image_to_clipboard(src: &str, opaque: bool) {
    let src = src.to_string();
    let opaque_str = if opaque { "true" } else { "false" };
    // Use spawn_forever so the task survives context menu unmount.
    // The menu closes (on_close) immediately after this call, which would
    // cancel a regular spawn task before eval.recv() completes.
    dioxus_core::spawn_forever(async move {
        // For HTTP URLs, download via Rust to bypass browser CORS restrictions.
        // Cross-origin images taint the Canvas, making toDataURL() throw SecurityError.
        let rasterize_src = if src.starts_with("http://") || src.starts_with("https://") {
            let (tx, rx) = tokio::sync::oneshot::channel();
            std::thread::spawn({
                let src = src.clone();
                move || {
                    let _ = tx.send(crate::utils::image::download_image_as_data_url(&src));
                }
            });
            match rx.await {
                Ok(Ok(data_url)) => data_url,
                Ok(Err(e)) => {
                    tracing::error!(%e, "Failed to download image for clipboard copy");
                    return;
                }
                Err(_) => {
                    tracing::error!("Image download thread was cancelled");
                    return;
                }
            }
        } else {
            src
        };

        let Ok(src_json) = serde_json::to_string(&rasterize_src) else {
            tracing::error!("Failed to serialize image src as JSON");
            return;
        };
        let js = format!(
            "(async () => {{ dioxus.send(await window.Arto.rasterize.image({}, {})); }})();",
            src_json, opaque_str
        );
        let mut eval = document::eval(&js);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            eval.recv::<Option<String>>(),
        )
        .await;
        match result {
            Ok(Ok(Some(data_url))) => {
                std::thread::spawn(move || {
                    crate::utils::clipboard::copy_image_from_data_url(&data_url);
                });
            }
            Ok(Ok(None)) => {
                tracing::warn!("Image rasterization returned null (image may have failed to load)");
            }
            Ok(Err(e)) => {
                tracing::error!(%e, "Failed to rasterize image via Canvas");
            }
            Err(_) => {
                tracing::error!("Image rasterization timed out");
            }
        }
    });
}

/// "Copy Image As..." submenu: Image / Image with Background / Markdown / Path
#[component]
pub(super) fn CopyImageAsSubmenu(
    src: String,
    alt_text: Option<String>,
    on_close: EventHandler<()>,
) -> Element {
    let alt_for_markdown = alt_text.unwrap_or_default();

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Image As...",

            ContextMenuItem {
                label: "Image",
                on_click: {
                    let src = src.clone();
                    move |_| {
                        copy_image_to_clipboard(&src, false);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Image with Background",
                on_click: {
                    let src = src.clone();
                    move |_| {
                        copy_image_to_clipboard(&src, true);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Markdown",
                on_click: {
                    let src = src.clone();
                    let alt_for_markdown = alt_for_markdown.clone();
                    move |_| {
                        let markdown = format!("![{}]({})", alt_for_markdown, src);
                        crate::utils::clipboard::copy_text(&markdown);
                        on_close.call(());
                    }
                },
            }

            super::menu_item::CopyMenuItem { label: "Path", text: src, on_close }
        }
    }
}

/// "Copy Image As..." submenu: Image / Image with Background
#[component]
pub(super) fn CopySpecialBlockAsSubmenu(is_mermaid: bool, on_close: EventHandler<()>) -> Element {
    rsx! {
        ContextMenuSubmenu {
            label: "Copy Image As...",

            ContextMenuItem {
                label: "Image",
                on_click: {
                    move |_| {
                        copy_special_block_to_clipboard(is_mermaid, false);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Image with Background",
                on_click: {
                    move |_| {
                        copy_special_block_to_clipboard(is_mermaid, true);
                        on_close.call(());
                    }
                },
            }
        }
    }
}

/// Copy a special block (Mermaid or Math) to clipboard via JS rasterization.
pub(super) fn copy_special_block_to_clipboard(is_mermaid: bool, opaque: bool) {
    rasterize_special_block(is_mermaid, opaque, |data_url| {
        std::thread::spawn(move || {
            crate::utils::clipboard::copy_image_from_data_url(&data_url);
        });
    });
}

/// Save a special block (Mermaid or Math) as an image file.
pub(super) fn save_special_block_as_image(is_mermaid: bool) {
    rasterize_special_block(is_mermaid, true, |data_url| {
        crate::utils::image::save_image(&data_url);
    });
}

/// Rasterize a special block (Mermaid or Math) via JS and invoke the callback
/// with the resulting data URL on success.
///
/// Uses `spawn_forever` so the task survives context menu unmount.
/// The menu closes (on_close) immediately after this call, which would
/// cancel a regular spawn task before eval.recv() completes.
pub(super) fn rasterize_special_block(
    is_mermaid: bool,
    opaque: bool,
    on_success: impl FnOnce(String) + Send + 'static,
) {
    let opaque_str = if opaque { "true" } else { "false" };
    let block_type = if is_mermaid { "Mermaid" } else { "Math" };
    dioxus_core::spawn_forever(async move {
        let js = if is_mermaid {
            format!(
                "(async () => {{ dioxus.send(await window.Arto.rasterize.mermaidBlock({})); }})();",
                opaque_str
            )
        } else {
            format!(
                "(async () => {{ dioxus.send(await window.Arto.rasterize.mathBlock({})); }})();",
                opaque_str
            )
        };

        let mut eval = document::eval(&js);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            eval.recv::<Option<String>>(),
        )
        .await;

        // Cleanup element references after rasterization
        let _ = document::eval("window.Arto.contextMenu.cleanup();");

        match result {
            Ok(Ok(Some(data_url))) => on_success(data_url),
            Ok(Ok(None)) => {
                tracing::warn!(block_type, "Block rasterization returned null");
            }
            Ok(Err(e)) => {
                tracing::error!(%e, block_type, "Failed to rasterize block");
            }
            Err(_) => {
                tracing::error!(block_type, "Block rasterization timed out");
            }
        }
    });
}
