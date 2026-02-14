use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::components::icon::{Icon, IconName};
use crate::state::AppState;

/// Context type for right-click detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentContext {
    /// General content (no specific element)
    General,
    /// Link element
    Link { href: String },
    /// Image element
    Image { src: String, alt: Option<String> },
    /// Code block
    CodeBlock {
        content: String,
        language: Option<String>,
        /// Block source line start (1-based, from data-source-line)
        #[serde(default)]
        source_line: Option<u32>,
        /// Block source line end (1-based, from data-source-line-end)
        #[serde(default)]
        source_line_end: Option<u32>,
    },
    /// Mermaid diagram
    Mermaid { source: String },
    /// Math block (display math or math code block)
    MathBlock { source: String },
}

/// Context menu data from JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMenuData {
    pub context: ContentContext,
    pub x: i32,
    pub y: i32,
    /// Whether there is selected text
    pub has_selection: bool,
    /// The selected text (captured at context menu open time)
    #[serde(default)]
    pub selected_text: String,
    /// Source line number at click/selection start position (1-based)
    #[serde(default)]
    pub source_line: Option<u32>,
    /// Source line number at selection end position (1-based, same as source_line for single line)
    #[serde(default)]
    pub source_line_end: Option<u32>,
    /// Table data as CSV (if right-clicked within a table)
    #[serde(default)]
    pub table_csv: Option<String>,
    /// Table data as TSV (if right-clicked within a table)
    #[serde(default)]
    pub table_tsv: Option<String>,
    /// Table source line start (1-based)
    #[serde(default)]
    pub table_source_line: Option<u32>,
    /// Table source line end (1-based)
    #[serde(default)]
    pub table_source_line_end: Option<u32>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ContentContextMenuProps {
    pub position: (i32, i32),
    pub context: ContentContext,
    pub has_selection: bool,
    pub selected_text: String,
    pub current_file: Option<PathBuf>,
    pub base_dir: PathBuf,
    pub source_line: Option<u32>,
    pub source_line_end: Option<u32>,
    pub table_csv: Option<String>,
    pub table_tsv: Option<String>,
    pub table_source_line: Option<u32>,
    pub table_source_line_end: Option<u32>,
    pub on_close: EventHandler<()>,
}

#[component]
pub fn ContentContextMenu(props: ContentContextMenuProps) -> Element {
    // Extract copyable source from context (code blocks, mermaid, math)
    let (copy_code_source, code_block_line, code_block_line_end) = match &props.context {
        ContentContext::CodeBlock {
            content,
            source_line,
            source_line_end,
            ..
        } => (Some(content.clone()), *source_line, *source_line_end),
        ContentContext::Mermaid { source } | ContentContext::MathBlock { source } => {
            // The renderer sets props.source_line/source_line_end to the block's
            // line range for mermaid/math (via detectContext's block-level override)
            (Some(source.clone()), props.source_line, props.source_line_end)
        }
        _ => (None, None, None),
    };

    // Extract image info for smart default and submenu
    let image_info = match &props.context {
        ContentContext::Image { src, alt } => Some((src.clone(), alt.clone())),
        _ => None,
    };
    let is_image = image_info.is_some();

    let has_context_specific = matches!(
        props.context,
        ContentContext::Link { .. } | ContentContext::Image { .. }
    );

    let has_table = props.table_csv.is_some();
    let has_file = props.current_file.is_some();
    let has_any_submenu =
        props.has_selection || has_file || copy_code_source.is_some() || has_table || is_image;

    // Determine smart default for Copy Path label and value
    let (copy_path_label, copy_path_value) = match (
        props.current_file.as_ref(),
        props.source_line,
        props.source_line_end,
    ) {
        (Some(f), Some(start), Some(end)) if start != end => {
            let path_str = f.display().to_string();
            (
                format!("Copy Path with Range ({start}-{end})"),
                Some(format!("{path_str}:{start}-{end}")),
            )
        }
        (Some(f), Some(line), _) => {
            let path_str = f.display().to_string();
            (
                format!("Copy Path with Line ({line})"),
                Some(format!("{path_str}:{line}")),
            )
        }
        (Some(f), None, _) => {
            let path_str = f.display().to_string();
            ("Copy Path".to_string(), Some(path_str))
        }
        (None, _, _) => ("Copy Path".to_string(), None),
    };

    rsx! {
        // Backdrop to close menu on outside click
        div {
            class: "context-menu-backdrop",
            // Prevent mousedown from clearing text selection
            onmousedown: move |evt| evt.prevent_default(),
            onclick: move |_| props.on_close.call(()),
        }

        // Context menu
        div {
            class: "context-menu content-context-menu",
            style: "left: {props.position.0}px; top: {props.position.1}px;",
            // Prevent mousedown from clearing text selection
            onmousedown: move |evt| evt.prevent_default(),
            onclick: move |evt| evt.stop_propagation(),

            // === Section 1: Smart default copy operations ===
            if props.has_selection {
                ContextMenuItem {
                    label: "Copy",
                    shortcut: Some("⌘C"),
                    icon: Some(IconName::Copy),
                    on_click: {
                        let selected_text = props.selected_text.clone();
                        let on_close = props.on_close;
                        move |_| {
                            crate::utils::clipboard::copy_text(&selected_text);
                            on_close.call(());
                        }
                    },
                }
            }

            if let Some(source) = copy_code_source.clone() {
                ContextMenuItem {
                    label: "Copy Code",
                    icon: Some(IconName::Copy),
                    on_click: {
                        let on_close = props.on_close;
                        move |_| {
                            crate::utils::clipboard::copy_text(&source);
                            on_close.call(());
                        }
                    },
                }
            }

            // Copy Table (smart default: TSV)
            if let Some(tsv) = props.table_tsv.clone() {
                ContextMenuItem {
                    label: "Copy Table",
                    icon: Some(IconName::Copy),
                    on_click: {
                        let on_close = props.on_close;
                        move |_| {
                            crate::utils::clipboard::copy_text(&tsv);
                            on_close.call(());
                        }
                    },
                }
            }

            // Copy Image (smart default)
            if let Some((ref src, _)) = image_info {
                ContextMenuItem {
                    label: "Copy Image",
                    icon: Some(IconName::Photo),
                    on_click: {
                        let src = src.clone();
                        let on_close = props.on_close;
                        move |_| {
                            crate::utils::clipboard::copy_image_from_data_url(&src);
                            on_close.call(());
                        }
                    },
                }
            }

            // Copy Path (smart default: path / path:line / path:start-end)
            if let Some(value) = copy_path_value.clone() {
                ContextMenuItem {
                    label: copy_path_label.clone(),
                    icon: Some(IconName::Copy),
                    on_click: {
                        let on_close = props.on_close;
                        move |_| {
                            crate::utils::clipboard::copy_text(&value);
                            on_close.call(());
                        }
                    },
                }
            }

            // === Section 2: Selection and search ===
            ContextMenuSeparator {}

            ContextMenuItem {
                label: "Select All",
                shortcut: Some("⌘A"),
                icon: Some(IconName::SelectAll),
                on_click: {
                    let on_close = props.on_close;
                    move |_| {
                        // Inject JS that schedules itself with setTimeout
                        // This runs after menu closes without needing async in Rust
                        let _ = document::eval(r#"
                            setTimeout(() => {
                                const el = document.querySelector('.markdown-body');
                                if (el) {
                                    const range = document.createRange();
                                    range.selectNodeContents(el);
                                    const selection = window.getSelection();
                                    selection.removeAllRanges();
                                    selection.addRange(range);
                                }
                            }, 50);
                        "#);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Find in Page",
                shortcut: Some("⌘F"),
                icon: Some(IconName::Search),
                on_click: {
                    let on_close = props.on_close;
                    let selected_text = props.selected_text.clone();
                    let has_selection = props.has_selection;
                    move |_| {
                        let mut state = use_context::<AppState>();
                        let text = if has_selection && !selected_text.is_empty() {
                            Some(selected_text.clone())
                        } else {
                            None
                        };
                        state.open_search_with_text(text);
                        on_close.call(());
                    }
                },
            }

            // === Section 3: Copy As... submenus ===
            if has_any_submenu {
                ContextMenuSeparator {}
            }

            // Copy As... (Text / Markdown)
            if props.has_selection {
                CopyAsSubmenu {
                    selected_text: props.selected_text.clone(),
                    current_file: props.current_file.clone(),
                    source_line: props.source_line,
                    source_line_end: props.source_line_end,
                    on_close: props.on_close,
                }
            }

            // Copy Path As... (Path / Path with Line / Path with Range)
            if has_file {
                CopyPathAsSubmenu {
                    current_file: props.current_file.clone().unwrap(),
                    source_line: props.source_line,
                    source_line_end: props.source_line_end,
                    on_close: props.on_close,
                }
            }

            // Copy Code As... (Code / Markdown / Path with Range)
            if let Some(code_source) = copy_code_source.clone() {
                CopyCodeAsSubmenu {
                    code_source,
                    current_file: props.current_file.clone(),
                    block_source_line: code_block_line,
                    block_source_line_end: code_block_line_end,
                    on_close: props.on_close,
                }
            }

            // Copy Table As... (TSV / CSV / Markdown)
            if has_table {
                CopyTableAsSubmenu {
                    table_tsv: props.table_tsv.clone(),
                    table_csv: props.table_csv.clone(),
                    current_file: props.current_file.clone(),
                    table_source_line: props.table_source_line,
                    table_source_line_end: props.table_source_line_end,
                    on_close: props.on_close,
                }
            }

            // Copy Image As... (Image / Markdown / Path)
            if let Some((ref src, ref alt_text)) = image_info {
                CopyImageAsSubmenu {
                    src: src.clone(),
                    alt_text: alt_text.clone(),
                    on_close: props.on_close,
                }
            }

            // === Section 4: Context-specific items (link, image) ===
            if has_context_specific {
                ContextMenuSeparator {}
            }

            match &props.context {
                ContentContext::Link { href } => rsx! {
                    LinkContextItems {
                        href: href.clone(),
                        base_dir: props.base_dir.clone(),
                        on_close: props.on_close,
                    }
                },
                ContentContext::Image { src, .. } => rsx! {
                    ImageContextItems {
                        src: src.clone(),
                        on_close: props.on_close,
                    }
                },
                _ => rsx! {},
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract markdown source lines from a file and copy to clipboard.
/// Runs file I/O on a background thread to avoid blocking the UI.
fn copy_markdown_source(file: &std::path::Path, start: u32, end: u32) {
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
fn copy_markdown_source_selection(
    file: &std::path::Path,
    start: u32,
    end: u32,
    selected_text: &str,
) {
    let file = file.to_path_buf();
    let selected_text = selected_text.to_string();
    std::thread::spawn(move || {
        let Some(source) =
            crate::utils::source_extract::extract_source_lines(&file, start, end)
        else {
            tracing::debug!(?file, start, end, "Failed to extract source lines");
            return;
        };
        // Map rendered selection back to markdown source substring
        let text = crate::utils::source_extract::extract_source_selection(
            &source,
            &selected_text,
        )
        .unwrap_or(source);
        crate::utils::clipboard::copy_text(&text);
    });
}

/// Build a "path:start-end" or "path:line" string from file path and line range.
/// Returns `None` if file or either line number is unavailable.
fn build_path_with_range(
    file: Option<&std::path::PathBuf>,
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

// ============================================================================
// Helper Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct ContextMenuItemProps {
    #[props(into)]
    label: String,
    #[props(default)]
    shortcut: Option<&'static str>,
    #[props(default)]
    icon: Option<IconName>,
    #[props(default = false)]
    disabled: bool,
    on_click: EventHandler<()>,
}

#[component]
fn ContextMenuItem(props: ContextMenuItemProps) -> Element {
    rsx! {
        div {
            class: if props.disabled { "context-menu-item disabled" } else { "context-menu-item" },
            onclick: move |_| {
                if !props.disabled {
                    props.on_click.call(());
                }
            },

            if let Some(icon) = props.icon {
                Icon {
                    name: icon,
                    size: 14,
                    class: "context-menu-icon",
                }
            }

            span { class: "context-menu-label", "{props.label}" }

            if let Some(shortcut) = props.shortcut {
                span { class: "context-menu-shortcut", "{shortcut}" }
            }
        }
    }
}

#[component]
fn ContextMenuSeparator() -> Element {
    rsx! {
        div { class: "context-menu-separator" }
    }
}

/// A menu item that copies text to clipboard and closes the menu.
/// Reduces boilerplate for the common "copy + close" pattern.
#[component]
fn CopyMenuItem(
    #[props(into)] label: String,
    #[props(into)] text: String,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        ContextMenuItem {
            label,
            on_click: move |_| {
                crate::utils::clipboard::copy_text(&text);
                on_close.call(());
            },
        }
    }
}

/// Reusable submenu component with hover-to-open behavior.
#[component]
fn ContextMenuSubmenu(label: String, children: Element) -> Element {
    let mut show = use_signal(|| false);

    rsx! {
        div {
            class: "context-menu-item has-submenu",
            onmouseenter: move |_| show.set(true),
            onmouseleave: move |_| show.set(false),

            span { class: "context-menu-label", "{label}" }
            span { class: "submenu-arrow", "›" }

            if *show.read() {
                div {
                    class: "context-submenu",
                    {children}
                }
            }
        }
    }
}

// ============================================================================
// Copy As... Submenu
// ============================================================================

/// "Copy As..." submenu: Text / Markdown
#[component]
fn CopyAsSubmenu(
    selected_text: String,
    current_file: Option<PathBuf>,
    source_line: Option<u32>,
    source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    // Show "Markdown" option when file and at least start line are known.
    let has_markdown_source = current_file.is_some() && source_line.is_some();
    let effective_end = source_line_end.or(source_line);

    rsx! {
        ContextMenuSubmenu {
            label: "Copy As...",

            CopyMenuItem { label: "Text", text: selected_text.clone(), on_close }

            if has_markdown_source {
                ContextMenuItem {
                    label: "Markdown",
                    on_click: {
                        let selected_text = selected_text.clone();
                        let current_file = current_file.clone();
                        move |_| {
                            // Extract the full source lines, then trim to the
                            // selected portion so partial selections don't copy
                            // entire lines.
                            let file = current_file.as_ref().unwrap();
                            let start = source_line.unwrap();
                            let end = effective_end.unwrap();
                            copy_markdown_source_selection(
                                file,
                                start,
                                end,
                                &selected_text,
                            );
                            on_close.call(());
                        }
                    },
                }
            }
        }
    }
}

// ============================================================================
// Copy Path As... Submenu
// ============================================================================

/// "Copy Path As..." submenu: Path / Path with Line / Path with Range
#[component]
fn CopyPathAsSubmenu(
    current_file: PathBuf,
    source_line: Option<u32>,
    source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let path_str = current_file.display().to_string();
    let has_range =
        source_line.is_some() && source_line_end.is_some() && source_line != source_line_end;

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Path As...",

            CopyMenuItem { label: "Path", text: path_str.clone(), on_close }

            if let Some(line) = source_line {
                CopyMenuItem {
                    label: format!("Path with Line ({line})"),
                    text: format!("{path_str}:{line}"),
                    on_close,
                }
            }

            if has_range {
                if let (Some(start), Some(end)) = (source_line, source_line_end) {
                    CopyMenuItem {
                        label: format!("Path with Range ({start}-{end})"),
                        text: format!("{path_str}:{start}-{end}"),
                        on_close,
                    }
                }
            }
        }
    }
}

// ============================================================================
// Copy Table As... Submenu
// ============================================================================

/// "Copy Table As..." submenu: TSV / CSV / Path with Range / Markdown
#[component]
fn CopyTableAsSubmenu(
    table_tsv: Option<String>,
    table_csv: Option<String>,
    current_file: Option<PathBuf>,
    table_source_line: Option<u32>,
    table_source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let has_markdown_source =
        current_file.is_some() && table_source_line.is_some() && table_source_line_end.is_some();

    let table_path_with_range = build_path_with_range(
        current_file.as_ref(),
        table_source_line,
        table_source_line_end,
    );

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Table As...",

            if let Some(tsv) = table_tsv {
                CopyMenuItem { label: "TSV", text: tsv, on_close }
            }

            if let Some(csv) = table_csv {
                CopyMenuItem { label: "CSV", text: csv, on_close }
            }

            if has_markdown_source {
                ContextMenuItem {
                    label: "Markdown",
                    on_click: {
                        let current_file = current_file.clone();
                        move |_| {
                            if let (Some(file), Some(start), Some(end)) =
                                (current_file.as_ref(), table_source_line, table_source_line_end)
                            {
                                copy_markdown_source(file, start, end);
                            }
                            on_close.call(());
                        }
                    },
                }
            }

            if let Some((path_value, start, end)) = table_path_with_range.clone() {
                CopyMenuItem {
                    label: if start != end {
                        format!("Path with Range ({start}-{end})")
                    } else {
                        format!("Path with Line ({start})")
                    },
                    text: path_value,
                    on_close,
                }
            }
        }
    }
}

// ============================================================================
// Copy Code As... Submenu
// ============================================================================

/// "Copy Code As..." submenu: Code / Markdown / Path with Range
#[component]
fn CopyCodeAsSubmenu(
    code_source: String,
    current_file: Option<PathBuf>,
    /// Block-level source line range (for Markdown and Path with Range)
    block_source_line: Option<u32>,
    block_source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let has_markdown =
        current_file.is_some() && block_source_line.is_some() && block_source_line_end.is_some();

    let block_path_with_range = build_path_with_range(
        current_file.as_ref(),
        block_source_line,
        block_source_line_end,
    );

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Code As...",

            CopyMenuItem { label: "Code", text: code_source.clone(), on_close }

            if has_markdown {
                ContextMenuItem {
                    label: "Markdown",
                    on_click: {
                        let current_file = current_file.clone();
                        move |_| {
                            if let (Some(file), Some(start), Some(end)) =
                                (current_file.as_ref(), block_source_line, block_source_line_end)
                            {
                                copy_markdown_source(file, start, end);
                            }
                            on_close.call(());
                        }
                    },
                }
            }

            if let Some((path_value, start, end)) = block_path_with_range.clone() {
                CopyMenuItem {
                    label: if start != end {
                        format!("Path with Range ({start}-{end})")
                    } else {
                        format!("Path with Line ({start})")
                    },
                    text: path_value,
                    on_close,
                }
            }
        }
    }
}

// ============================================================================
// Context-Specific Menu Items
// ============================================================================

#[component]
fn LinkContextItems(href: String, base_dir: PathBuf, on_close: EventHandler<()>) -> Element {
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

#[component]
fn ImageContextItems(src: String, on_close: EventHandler<()>) -> Element {
    rsx! {
        ContextMenuItem {
            label: "Save Image As...",
            icon: Some(IconName::Download),
            on_click: {
                let src = src.clone();
                let on_close = on_close;
                move |_| {
                    // Run in background thread to prevent UI blocking during HTTP download
                    let src = src.clone();
                    std::thread::spawn(move || {
                        crate::utils::image::save_image(&src);
                    });
                    on_close.call(());
                }
            },
        }
    }
}

// ============================================================================
// Copy Image As... Submenu
// ============================================================================

/// "Copy Image As..." submenu: Image / Markdown / Path
#[component]
fn CopyImageAsSubmenu(
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
                        crate::utils::clipboard::copy_image_from_data_url(&src);
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

            CopyMenuItem { label: "Path", text: src, on_close }
        }
    }
}
