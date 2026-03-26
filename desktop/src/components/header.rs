use dioxus::prelude::*;

use crate::components::bookmark_button::BookmarkButton;
use crate::components::icon::{Icon, IconName};
use crate::components::theme_selector::ThemeSelector;
use crate::state::AppState;

#[component]
pub fn Header() -> Element {
    let mut state = use_context::<AppState>();

    let mut is_menu_open = use_signal(|| false);

    let current_tab = state.current_tab();
    let file_path = current_tab.as_ref().and_then(|tab| tab.file());
    let file = file_path
        .as_ref()
        .map(|f| {
            f.file_name()
                .unwrap_or(f.as_os_str())
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| "No file opened".to_string());

    let can_go_back = current_tab
        .as_ref()
        .is_some_and(|tab| tab.history.can_go_back());
    let can_go_forward = current_tab
        .as_ref()
        .is_some_and(|tab| tab.history.can_go_forward());

    let on_back = move |_| {
        state.save_scroll_and_go_back();
    };

    let on_forward = move |_| {
        state.save_scroll_and_go_forward();
    };

    let is_reloading = use_signal(|| false);
    let mut is_reloading_write = is_reloading;

    let on_reload = move |_| {
        // Set reloading state
        is_reloading_write.set(true);

        state.reload_current_tab();

        // Reset reloading state after animation
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            is_reloading_write.set(false);
        });
    };

    // Copy feedback state
    let mut is_copied = use_signal(|| false);

    rsx! {
        div {
            class: "header",
            style: "position: relative;",

            // File name display (left side) with navigation buttons
            div {
                class: "header-left",

                    // Hamburger Menu Button
                    if cfg!(target_os = "windows") {
                        button {
                            class: "nav-button hamburger-button",
                            class: if *is_menu_open.read() { "active" },
                            title: "Menu",
                            onclick: move |_| is_menu_open.toggle(),
                            Icon { name: IconName::Menu2 }
                        }
                    }


                // Back button
                button {
                    class: "nav-button",
                    disabled: !can_go_back,
                    onclick: on_back,
                    Icon { name: IconName::ChevronLeft }
                }

                // Forward button
                button {
                    class: "nav-button",
                    disabled: !can_go_forward,
                    onclick: on_forward,
                    Icon { name: IconName::ChevronRight }
                }

                // File name
                span {
                    class: "file-name",
                    "{file}"
                }

                div {
                    class: "file-action-buttons",

                    // Bookmark, copy path, and reload buttons (shown on hover)
                    if let Some(path) = file_path {
                        // Bookmark button
                        BookmarkButton { path: path.to_path_buf() }

                        button {
                            class: "nav-button copy-button",
                            class: if *is_copied.read() { "copied" },
                            title: "Copy full path",
                            onclick: {
                                let path_str = path.to_string_lossy().to_string();
                                move |_| {
                                    crate::utils::clipboard::copy_text(&path_str);
                                    // Show success feedback
                                    is_copied.set(true);
                                    spawn(async move {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                        is_copied.set(false);
                                    });
                                }
                            },
                            Icon {
                                name: if *is_copied.read() { IconName::Check } else { IconName::Copy },
                                size: 14,
                            }
                        }

                        // Reload button (next to copy button)
                        button {
                            class: "nav-button reload-button",
                            class: if *is_reloading.read() { "reloading" },
                            onclick: on_reload,
                            title: "Reload file",
                            Icon { name: IconName::Refresh, size: 14 }
                        }
                    }
                }
            }

            // Right side controls
            div {
                class: "header-right",

                // Search button
                button {
                    class: "nav-button search-button",
                    class: if *state.search_open.read() { "active" },
                    title: "Search in page",
                    onclick: move |_| {
                        let was_closed = !*state.search_open.read();
                        state.toggle_search();
                        if was_closed {
                            // Focus the search input after opening
                            spawn(async {
                                let _ = document::eval(
                                    "document.querySelector('.search-input')?.focus()",
                                )
                                .await;
                            });
                        }
                    },
                    Icon { name: IconName::Search, size: 20 }
                }

                // Theme selector
                ThemeSelector { current_theme: state.current_theme }
            }

            if *is_menu_open.read() {
                AppMenu {
                    on_close: move |_| is_menu_open.set(false),
                }
            }

        }
    }
}

// Hamburger menu for Windows

#[component]
fn AppMenu(on_close: EventHandler<()>) -> Element {
    let mut state = use_context::<AppState>();

    // Helper to get keyboard shortcut hints
    let shortcut = |action| crate::keybindings::shortcut_hint_for_global_action(action);

    // Get information on the currently open file (for invalidation determination)
    let current_tab = state.current_tab();
    let current_file = current_tab
        .as_ref()
        .and_then(|t| t.file().map(|f| f.to_path_buf()));
    let has_file = current_file.is_some();

    let close = move || on_close.call(());

    rsx! {
        // Transparent background to close when clicking outside menu
        div {
            class: "context-menu-backdrop",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 998;",
            onclick: move |_| close(),
        }

        // Menu body
        div {
            class: "context-menu",
            style: "position: absolute; left: 12px; top: 40px; z-index: 999;",
            onclick: move |evt| evt.stop_propagation(),

            // === Arto (App) ===
            HeaderMenuItem { label: "About Arto", shortcut: shortcut("app.about"), on_click: move |_| {
                crate::components::content::set_preferences_tab_to_about();
                state.open_preferences();
                close();
            } }
            HeaderMenuItem { label: "Preferences...", shortcut: shortcut("file.preferences"), icon: Some(IconName::Gear), on_click: move |_| {
                state.open_preferences();
                close();
            } }

            HeaderMenuSeparator {}

            // === File ===
            HeaderSubmenu { label: "File",
                HeaderMenuItem { label: "New Window", shortcut: shortcut("window.new"), on_click: move |_| {
                    crate::window::create_main_window_sync(&dioxus::desktop::window(), crate::state::Tab::default(), crate::window::CreateMainWindowConfigParams::default());
                    close();
                } }
                HeaderMenuItem { label: "New Tab", shortcut: shortcut("tab.new"), icon: Some(IconName::Add), on_click: move |_| {
                    state.add_empty_tab(true);
                    close();
                } }
                HeaderMenuSeparator {}
                HeaderMenuItem { label: "Open File...", shortcut: shortcut("file.open"), icon: Some(IconName::File), on_click: move |_| {
                    if let Some(file) = rfd::FileDialog::new().add_filter("Markdown", &["md", "markdown"]).pick_file() {
                        state.open_file(file);
                    }
                    close();
                } }
                HeaderMenuItem { label: "Open Directory...", shortcut: shortcut("file.open_directory"), icon: Some(IconName::FolderOpen), on_click: move |_| {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        state.set_root_directory(dir);
                    }
                    close();
                } }
                HeaderMenuSeparator {}
                HeaderMenuItem { label: "Copy File Path", shortcut: shortcut("clipboard.copy_file_path"), icon: Some(IconName::Copy), disabled: !has_file, on_click: { let f = current_file.clone(); move |_| {
                    if let Some(file) = &f { crate::utils::clipboard::copy_text(file.to_string_lossy()); }
                    close();
                } } }
                HeaderMenuItem { label: "Reveal in Finder", shortcut: shortcut("file.reveal_in_finder"), icon: Some(IconName::Folder), disabled: !has_file, on_click: { let f = current_file.clone(); move |_| {
                    if let Some(file) = &f { crate::utils::file_operations::reveal_in_finder(file); }
                    close();
                } } }
                HeaderMenuSeparator {}
                HeaderMenuItem { label: "Close Tab", shortcut: shortcut("tab.close"), on_click: move |_| {
                    let active = *state.active_tab.read();
                    state.close_tab(active);
                    close();
                } }
                HeaderMenuItem { label: "Close All Tabs", shortcut: shortcut("tab.close_all"), on_click: move |_| {
                    let mut tabs = state.tabs.write();
                    tabs.clear();
                    tabs.push(crate::state::Tab::default());
                    state.active_tab.set(0);
                    close();
                } }
                HeaderMenuItem { label: "Close Window", shortcut: shortcut("window.close"), on_click: move |_| {
                    dioxus::desktop::window().close();
                } }
            }

            // === Edit ===
            HeaderSubmenu { label: "Edit",
                HeaderMenuItem { label: "Find...", shortcut: shortcut("search.open"), icon: Some(IconName::Search), on_click: move |_| {
                    state.open_search_with_text(None);
                    close();
                } }
                HeaderMenuItem { label: "Find Next", shortcut: shortcut("search.next"), on_click: move |_| {
                    spawn(async move { let _ = document::eval("window.Arto.search.navigate('next')").await; });
                    close();
                } }
                HeaderMenuItem { label: "Find Previous", shortcut: shortcut("search.prev"), on_click: move |_| {
                    spawn(async move { let _ = document::eval("window.Arto.search.navigate('prev')").await; });
                    close();
                } }
            }

            // === View ===
            HeaderSubmenu { label: "View",
                HeaderMenuItem { label: "Toggle Left Sidebar", shortcut: shortcut("window.toggle_sidebar"), icon: Some(IconName::Sidebar), on_click: move |_| {
                    state.toggle_sidebar();
                    close();
                } }
                HeaderMenuItem { label: "Toggle Right Sidebar", shortcut: shortcut("window.toggle_right_sidebar"), icon: Some(IconName::List), on_click: move |_| {
                    state.toggle_right_sidebar();
                    close();
                } }
                HeaderMenuSeparator {}
                HeaderMenuItem { label: "Actual Size", shortcut: shortcut("zoom.reset"), on_click: move |_| {
                    state.zoom_level.set(1.0);
                    close();
                } }
                HeaderMenuItem { label: "Zoom In", shortcut: shortcut("zoom.in"), icon: Some(IconName::Add), on_click: move |_| {
                    let current = crate::window::settings::normalize_zoom_level(*state.zoom_level.read());
                    state.zoom_level.set(crate::window::settings::normalize_zoom_level(current + 0.1));
                    close();
                } }
                HeaderMenuItem { label: "Zoom Out", shortcut: shortcut("zoom.out"), on_click: move |_| {
                    let current = crate::window::settings::normalize_zoom_level(*state.zoom_level.read());
                    state.zoom_level.set(crate::window::settings::normalize_zoom_level(current - 0.1));
                    close();
                } }
            }

            // === History ===
            HeaderSubmenu { label: "History",
                HeaderMenuItem { label: "Go Back", shortcut: shortcut("history.back"), icon: Some(IconName::ChevronLeft), on_click: move |_| {
                    state.save_scroll_and_go_back();
                    close();
                } }
                HeaderMenuItem { label: "Go Forward", shortcut: shortcut("history.forward"), icon: Some(IconName::ChevronRight), on_click: move |_| {
                    state.save_scroll_and_go_forward();
                    close();
                } }
            }

            // === Window ===
            HeaderSubmenu { label: "Window",
                HeaderMenuItem { label: "Close All Child Windows", shortcut: shortcut("window.close_all_child_windows"), on_click: move |_| {
                    crate::window::close_child_windows_for_last_focused();
                    close();
                } }
                HeaderMenuItem { label: "Close All Windows", shortcut: shortcut("window.close_all_windows"), on_click: move |_| {
                    crate::window::close_all_main_windows();
                    close();
                } }
            }

            // === Help ===
            HeaderSubmenu { label: "Help",
                HeaderMenuItem { label: "Go to Homepage", shortcut: shortcut("app.go_to_homepage"), icon: Some(IconName::ExternalLink), on_click: move |_| {
                    let _ = open::that("https://github.com/arto-app/Arto");
                    close();
                } }
            }

            HeaderMenuSeparator {}

            // === Quit ===
            HeaderMenuItem { label: "Quit", icon: Some(IconName::Close), on_click: move |_| {
                crate::window::shutdown_all_windows();
            } }
        }
    }
}

// Component for expanding submenus
#[component]
fn HeaderSubmenu(#[props(into)] label: String, children: Element) -> Element {
    let mut show = use_signal(|| false);

    rsx! {
        div {
            class: "context-menu-item has-submenu",
            onmouseenter: move |_| show.set(true),
            onmouseleave: move |_| show.set(false),

            span { class: "context-menu-label", "{label}" }
            span { class: "submenu-arrow", "›" }

            if *show.read() {
                div {
                    class: "context-submenu",
                    {children}
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct HeaderMenuItemProps {
    #[props(into)]
    label: String,
    #[props(default)]
    icon: Option<IconName>,
    #[props(default)]
    shortcut: Option<String>,
    #[props(default = false)]
    disabled: bool,
    on_click: EventHandler<()>,
}

#[component]
fn HeaderMenuItem(props: HeaderMenuItemProps) -> Element {
    let on_click = props.on_click;
    let disabled = props.disabled;

    rsx! {
        div {
            class: if disabled { "context-menu-item disabled" } else { "context-menu-item" },
            onclick: move |_| {
                if !disabled {
                    on_click.call(());
                }
            },

            if let Some(icon) = props.icon {
                Icon {
                    name: icon,
                    size: 14,
                    class: "context-menu-icon",
                }
            }
            span { class: "context-menu-label", "{props.label}" }

            if let Some(shortcut) = props.shortcut {
                span { class: "context-menu-shortcut", "{shortcut}" }
            }
        }
    }
}

#[component]
fn HeaderMenuSeparator() -> Element {
    rsx! {
        div { class: "context-menu-separator" }
    }
}
