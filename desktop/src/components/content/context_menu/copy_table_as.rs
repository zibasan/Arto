use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu, CopyMenuItem};
use super::source_ops::{build_path_with_range, copy_markdown_source};

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
