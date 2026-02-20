use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
use dioxus::desktop::tao::window::WindowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OpenTarget<T> {
    ExistingWindow(T),
    NewWindow,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WindowSelectionInput<T> {
    pub(super) window_id: T,
    pub(super) is_on_current_screen: bool,
}

pub(super) fn choose_open_target<T: Copy + Eq>(
    behavior: crate::config::FileOpenBehavior,
    visible_windows: &[WindowSelectionInput<T>],
    last_focused: Option<T>,
) -> OpenTarget<T> {
    match behavior {
        crate::config::FileOpenBehavior::NewWindow => OpenTarget::NewWindow,
        crate::config::FileOpenBehavior::LastFocused => match last_focused {
            Some(window_id)
                if visible_windows
                    .iter()
                    .any(|window| window.window_id == window_id) =>
            {
                OpenTarget::ExistingWindow(window_id)
            }
            _ => OpenTarget::NewWindow,
        },
        crate::config::FileOpenBehavior::CurrentScreen => {
            let current_screen_windows: Vec<T> = visible_windows
                .iter()
                .filter(|window| window.is_on_current_screen)
                .map(|window| window.window_id)
                .collect();

            if current_screen_windows.is_empty() {
                return OpenTarget::NewWindow;
            }

            if let Some(window_id) = last_focused {
                if current_screen_windows.contains(&window_id) {
                    return OpenTarget::ExistingWindow(window_id);
                }
            }

            OpenTarget::ExistingWindow(current_screen_windows[0])
        }
    }
}

pub(super) fn select_target_window() -> Option<WindowId> {
    let behavior = crate::config::CONFIG.read().file_open;
    select_target_window_with_behavior(behavior)
}

pub(super) fn select_target_window_with_behavior(
    behavior: crate::config::FileOpenBehavior,
) -> Option<WindowId> {
    let cursor_display_bounds = crate::utils::screen::get_current_display_bounds();
    let visible_windows: Vec<WindowSelectionInput<WindowId>> =
        collect_visible_window_selection_inputs(cursor_display_bounds);
    let last_focused = crate::window::main::get_last_focused_window();

    match choose_open_target(behavior, &visible_windows, last_focused) {
        OpenTarget::ExistingWindow(window_id) => Some(window_id),
        OpenTarget::NewWindow => None,
    }
}

fn collect_visible_window_selection_inputs(
    current_display_bounds: Option<(LogicalPosition<i32>, LogicalSize<u32>)>,
) -> Vec<WindowSelectionInput<WindowId>> {
    crate::window::main::list_visible_main_windows()
        .into_iter()
        .map(|ctx| {
            let scale_factor = ctx.window.scale_factor();
            // Use consistent outer bounds (outer_position + outer_size) for display overlap detection
            let window_position = ctx
                .window
                .outer_position()
                .map(|p| LogicalPosition::from_physical(p, scale_factor))
                .unwrap_or_else(|_| {
                    let metrics = crate::window::metrics::capture_window_metrics(&ctx.window);
                    LogicalPosition::new(metrics.position.x, metrics.position.y)
                });
            let window_size = ctx.window.outer_size().to_logical(scale_factor);
            let is_on_current_screen = current_display_bounds
                .map(|display| is_window_on_display(display, window_position, window_size))
                .unwrap_or(false);
            WindowSelectionInput {
                window_id: ctx.window.id(),
                is_on_current_screen,
            }
        })
        .collect()
}

pub(super) fn is_window_on_display(
    display_bounds: (LogicalPosition<i32>, LogicalSize<u32>),
    window_position: LogicalPosition<i32>,
    window_size: LogicalSize<u32>,
) -> bool {
    let (display_origin, display_size) = display_bounds;
    let display_left = display_origin.x;
    let display_top = display_origin.y;
    let display_right = display_left + display_size.width as i32;
    let display_bottom = display_top + display_size.height as i32;

    let window_left = window_position.x;
    let window_top = window_position.y;
    let window_right = window_left + window_size.width as i32;
    let window_bottom = window_top + window_size.height as i32;

    window_left < display_right
        && display_left < window_right
        && window_top < display_bottom
        && display_top < window_bottom
}
