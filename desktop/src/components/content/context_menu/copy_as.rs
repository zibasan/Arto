use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu, CopyMenuItem};
use super::source_ops::copy_markdown_source_selection;

/// "Copy As..." submenu: Text / Markdown
#[component]
pub(super) fn CopyAsSubmenu(
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
