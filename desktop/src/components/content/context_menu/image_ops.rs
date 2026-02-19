use dioxus::prelude::*;

use super::menu_item::{ContextMenuItem, ContextMenuSubmenu};
use crate::keybindings::dispatcher::dispatch_action;
use crate::keybindings::Action;
use crate::keybindings::{shortcut_hint_for_context_action, KeyContext};
use crate::state::AppState;

/// "Copy Image As..." submenu: Image / Image with Background / Markdown / Path
#[component]
pub(super) fn CopyImageAsSubmenu(on_close: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let shortcut = |action| shortcut_hint_for_context_action(KeyContext::Content, action);

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Image As...",

            ContextMenuItem {
                label: "Image",
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyImage, state);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Image with Background",
                shortcut: shortcut("clipboard.copy_image_with_background"),
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyImageWithBackground, state);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Markdown",
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyImageAsMarkdown, state);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Path",
                shortcut: shortcut("clipboard.copy_image_path"),
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyImagePath, state);
                        on_close.call(());
                    }
                },
            }
        }
    }
}

/// "Copy Image As..." submenu: Image / Image with Background
#[component]
pub(super) fn CopySpecialBlockAsSubmenu(on_close: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let shortcut = |action| shortcut_hint_for_context_action(KeyContext::Content, action);

    rsx! {
        ContextMenuSubmenu {
            label: "Copy Image As...",

            ContextMenuItem {
                label: "Image",
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyImage, state);
                        on_close.call(());
                    }
                },
            }

            ContextMenuItem {
                label: "Image with Background",
                shortcut: shortcut("clipboard.copy_image_with_background"),
                on_click: {
                    move |_| {
                        dispatch_action(&Action::CopyImageWithBackground, state);
                        on_close.call(());
                    }
                },
            }
        }
    }
}
