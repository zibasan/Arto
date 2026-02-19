use dioxus::prelude::*;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu};
use crate::keybindings::dispatcher::dispatch_action;
use crate::keybindings::Action;

/// "Copy Path As..." submenu: Path / Path with Line / Path with Range
#[component]
pub(super) fn CopyPathAsSubmenu(
    source_line: Option<u32>,
    source_line_end: Option<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let state = use_context::<crate::state::AppState>();
    let has_range =
        source_line.is_some() && source_line_end.is_some() && source_line != source_line_end;

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Path As...",

            ContextMenuItem {
                label: "Path",
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyFilePath, state);
                        on_close.call(());
                    }
                },
            }

            if let Some(line) = source_line {
                ContextMenuItem {
                    label: format!("Path with Line ({line})"),
                    on_click: {
                        move |_| {
                            dispatch_action(&Action::CopyFilePathWithLine, state);
                            on_close.call(());
                        }
                    },
                }
            }

            if has_range {
                if let (Some(start), Some(end)) = (source_line, source_line_end) {
                    ContextMenuItem {
                        label: format!("Path with Range ({start}-{end})"),
                        on_click: {
                            move |_| {
                                dispatch_action(&Action::CopyFilePathWithRange, state);
                                on_close.call(());
                            }
                        },
                    }
                }
            }
        }
    }
}
