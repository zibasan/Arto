use dioxus::prelude::*;
use std::path::PathBuf;

use super::menu_item::{ContextMenuSubmenu, CopyMenuItem};

/// "Copy Path As..." submenu: Path / Path with Line / Path with Range
#[component]
pub(super) fn CopyPathAsSubmenu(
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
