use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu, CopyMenuItem};
use super::source_ops::{build_path_with_range, copy_markdown_source};

/// "Copy Code As..." submenu: Code / Markdown / Path with Range
#[component]
pub(super) fn CopyCodeAsSubmenu(
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
