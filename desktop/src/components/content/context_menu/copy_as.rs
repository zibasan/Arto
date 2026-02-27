use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu};
use crate::keybindings::dispatcher::dispatch_action;
use crate::keybindings::Action;

/// "Copy As..." submenu: Text / Markdown
#[component]
pub(super) fn CopyAsSubmenu(
    selected_text: String,
    current_file: Option<PathBuf>,
    source_line: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let state = use_context::<crate::state::AppState>();
    // Show "Markdown" option when file and at least start line are known.
    let has_markdown_source = current_file.is_some() && source_line.is_some();

    rsx! {
        ContextMenuSubmenu {
            label: "Copy As...",

            ContextMenuItem {
                label: "Text",
                on_click: {
                    let text = selected_text.clone();
                    move |_| {
                        crate::utils::clipboard::copy_text(&text);
                        on_close.call(());
                    }
                },
            }

            if has_markdown_source {
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
        }
    }
}
