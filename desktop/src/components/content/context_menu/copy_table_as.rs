use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu};
use super::source_ops::build_path_with_range;
use crate::keybindings::dispatcher::dispatch_action;
use crate::keybindings::Action;

/// "Copy Table As..." submenu: TSV / CSV / Path with Range / Markdown
#[component]
pub(super) fn CopyTableAsSubmenu(
    table_tsv: Option<String>,
    table_csv: Option<String>,
    current_file: Option<PathBuf>,
    table_source_line: Option<u32>,
    table_source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let state = use_context::<crate::state::AppState>();
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

            if let Some(_tsv) = table_tsv {
                ContextMenuItem {
                    label: "TSV",
                    on_click: {
                        move |_| {
                            dispatch_action(&Action::CopyTableAsTsv, state);
                            on_close.call(());
                        }
                    },
                }
            }

            if let Some(_csv) = table_csv {
                ContextMenuItem {
                    label: "CSV",
                    on_click: {
                        move |_| {
                            dispatch_action(&Action::CopyTableAsCsv, state);
                            on_close.call(());
                        }
                    },
                }
            }

            if has_markdown_source {
                ContextMenuItem {
                    label: "Markdown",
                    on_click: {
                        move |_| {
                            dispatch_action(&Action::CopyTableAsMarkdown, state);
                            on_close.call(());
                        }
                    },
                }
            }

            if let Some((_path_value, start, end)) = table_path_with_range.clone() {
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
