use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu};
use super::source_ops::build_path_with_range;
use crate::keybindings::dispatcher::dispatch_action;
use crate::keybindings::Action;

/// "Copy Code As..." submenu: Code / Markdown / Path with Range
#[component]
pub(super) fn CopyCodeAsSubmenu(
    current_file: Option<PathBuf>,
    /// Block-level source line range (for Markdown and Path with Range)
    block_source_line: Option<u32>,
    block_source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let state = use_context::<crate::state::AppState>();
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

            ContextMenuItem {
                label: "Code",
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyCode, state);
                        on_close.call(());
                    }
                },
            }

            if has_markdown {
                ContextMenuItem {
                    label: "Markdown",
                    on_click: {
                        move |_| {
                            dispatch_action(&Action::CopyAsMarkdown, state);
                            on_close.call(());
                        }
                    },
                }
            }

            if let Some((_path_value, start, end)) = block_path_with_range.clone() {
                ContextMenuItem {
                    label: if start != end {
                        format!("Path with Range ({start}-{end})")
                    } else {
                        format!("Path with Line ({start})")
                    },
                    on_click: {
                        move |_| {
                            let action = if start != end {
                                Action::CopyFilePathWithRange
                            } else {
                                Action::CopyFilePathWithLine
                            };
                            dispatch_action(&action, state);
                            on_close.call(());
                        }
                    },
                }
            }
        }
    }
}
