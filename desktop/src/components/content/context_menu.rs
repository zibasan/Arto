mod copy_as;
mod copy_code_as;
mod copy_path_as;
mod copy_table_as;
mod data;
mod image_ops;
mod menu_item;
mod source_ops;

pub use data::*;

use dioxus::prelude::*;

use crate::components::icon::IconName;
use crate::state::AppState;
use copy_as::CopyAsSubmenu;
use copy_code_as::CopyCodeAsSubmenu;
use copy_path_as::CopyPathAsSubmenu;
use copy_table_as::CopyTableAsSubmenu;
use image_ops::{
    copy_image_to_clipboard, copy_special_block_to_clipboard, save_special_block_as_image,
    CopyImageAsSubmenu, CopySpecialBlockAsSubmenu,
};
use menu_item::{ContextMenuItem, ContextMenuSeparator};
use source_ops::LinkContextItems;

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
            (
                Some(source.clone()),
                props.source_line,
                props.source_line_end,
            )
        }
        _ => (None, None, None),
    };

    // Extract image info for smart default and submenu
    let image_info = match &props.context {
        ContentContext::Image { src, alt } => Some((src.clone(), alt.clone())),
        _ => None,
    };
    let is_image = image_info.is_some();

    // Detect special blocks (Mermaid/Math) for image operations
    let (is_mermaid, is_math) = match &props.context {
        ContentContext::Mermaid { .. } => (true, false),
        ContentContext::MathBlock { .. } => (false, true),
        _ => (false, false),
    };
    let is_special_block = is_mermaid || is_math;

    let has_context_specific = matches!(
        props.context,
        ContentContext::Link { .. }
            | ContentContext::Image { .. }
            | ContentContext::Mermaid { .. }
            | ContentContext::MathBlock { .. }
    );

    let has_table = props.table_csv.is_some();
    let has_file = props.current_file.is_some();
    let has_any_submenu = props.has_selection
        || has_file
        || copy_code_source.is_some()
        || has_table
        || is_image
        || is_special_block;

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
            onclick: move |_| {
                props.on_close.call(());
            },
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

            // Copy Image (smart default: transparent background)
            if let Some((ref src, _)) = image_info {
                ContextMenuItem {
                    label: "Copy Image",
                    icon: Some(IconName::Photo),
                    on_click: {
                        let src = src.clone();
                        let on_close = props.on_close;
                        move |_| {
                            copy_image_to_clipboard(&src, false);
                            on_close.call(());
                        }
                    },
                }
            }

            // Copy Image for Mermaid/Math blocks (default: transparent)
            if is_special_block {
                ContextMenuItem {
                    label: "Copy Image",
                    icon: Some(IconName::Photo),
                    on_click: {
                        let on_close = props.on_close;
                        move |_| {
                            copy_special_block_to_clipboard(is_mermaid, false);
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

            // Copy Image As... (Image / Image with Background) for special blocks
            if is_special_block {
                CopySpecialBlockAsSubmenu {
                    is_mermaid,
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
                    ContextMenuItem {
                        label: "Save Image As...",
                        icon: Some(IconName::Download),
                        on_click: {
                            let src = src.clone();
                            let on_close = props.on_close;
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
                },
                ContentContext::Mermaid { .. } | ContentContext::MathBlock { .. } => rsx! {
                    ContextMenuItem {
                        label: "Save Image As...",
                        icon: Some(IconName::Download),
                        on_click: {
                            let on_close = props.on_close;
                            move |_| {
                                save_special_block_as_image(is_mermaid);
                                on_close.call(());
                            }
                        },
                    }
                },
                _ => rsx! {},
            }
        }
    }
}
