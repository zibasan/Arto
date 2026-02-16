use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::ContextMenuItem;
use crate::components::icon::IconName;
use crate::state::AppState;

/// Extract markdown source lines from a file and copy to clipboard.
/// Runs file I/O on a background thread to avoid blocking the UI.
pub(super) fn copy_markdown_source(file: &std::path::Path, start: u32, end: u32) {
    let file = file.to_path_buf();
    std::thread::spawn(move || {
        match crate::utils::source_extract::extract_source_lines(&file, start, end) {
            Some(md) => crate::utils::clipboard::copy_text(&md),
            None => tracing::debug!(?file, start, end, "Failed to extract source lines"),
        }
    });
}

/// Extract markdown source for a text selection and copy to clipboard.
///
/// Uses source line extraction + rendered→source position mapping to find
/// the exact markdown source substring corresponding to the user's selection.
/// Includes surrounding formatting markers (e.g., `**bold**` not just `bold`).
pub(super) fn copy_markdown_source_selection(
    file: &std::path::Path,
    start: u32,
    end: u32,
    selected_text: &str,
) {
    let file = file.to_path_buf();
    let selected_text = selected_text.to_string();
    std::thread::spawn(move || {
        let Some(source) = crate::utils::source_extract::extract_source_lines(&file, start, end)
        else {
            tracing::debug!(?file, start, end, "Failed to extract source lines");
            return;
        };
        // Map rendered selection back to markdown source substring
        let text = crate::utils::source_extract::extract_source_selection(&source, &selected_text)
            .unwrap_or(source);
        crate::utils::clipboard::copy_text(&text);
    });
}

/// Build a "path:start-end" or "path:line" string from file path and line range.
/// Returns `None` if file or either line number is unavailable.
pub(super) fn build_path_with_range(
    file: Option<&PathBuf>,
    start: Option<u32>,
    end: Option<u32>,
) -> Option<(String, u32, u32)> {
    let f = file?;
    let start = start?;
    let end = end?;
    let path_str = f.display().to_string();
    let formatted = if start != end {
        format!("{path_str}:{start}-{end}")
    } else {
        format!("{path_str}:{start}")
    };
    Some((formatted, start, end))
}

#[component]
pub(super) fn LinkContextItems(
    href: String,
    base_dir: PathBuf,
    on_close: EventHandler<()>,
) -> Element {
    let mut state = use_context::<AppState>();
    let target_path = base_dir.join(&href);

    rsx! {
        ContextMenuItem {
            label: "Open Link",
            icon: Some(IconName::ExternalLink),
            on_click: {
                let target_path = target_path.clone();
                let on_close = on_close;
                move |_| {
                    if let Ok(canonical) = target_path.canonicalize() {
                        state.navigate_to_file(canonical);
                    }
                    on_close.call(());
                }
            },
        }

        ContextMenuItem {
            label: "Open Link in New Tab",
            icon: Some(IconName::Add),
            on_click: {
                let target_path = target_path.clone();
                let on_close = on_close;
                move |_| {
                    if let Ok(canonical) = target_path.canonicalize() {
                        state.add_file_tab(canonical, true);
                    }
                    on_close.call(());
                }
            },
        }

        ContextMenuItem {
            label: "Copy Link Path",
            icon: Some(IconName::Copy),
            on_click: {
                let href = href.clone();
                let on_close = on_close;
                move |_| {
                    crate::utils::clipboard::copy_text(&href);
                    on_close.call(());
                }
            },
        }
    }
}
