//! Single-instance IPC module
//!
//! This module provides IPC functionality to ensure only one instance of Arto runs at a time.
//! When a second instance is launched with paths, it sends those paths to the existing instance
//! via a local socket and exits.
//!
//! # Architecture
//!
//! ```text
//! 1st Instance (Primary):
//!   main() → try_send_to_existing() fails → start_ipc_server()
//!                                                 ↓
//!                                           accept connections
//!                                                 ↓
//!                                           recv JSON Lines → IPC_EVENT_QUEUE
//!                                                 ↓
//!                                           GCD wake → process_pending_events()
//!
//! 2nd Instance (Secondary):
//!   main() → try_send_to_existing() succeeds → exit(0)
//! ```
//!
//! # Protocol
//!
//! Messages are sent as JSON Lines (one JSON object per line):
//!
//! ```json
//! {"type":"file","path":"/path/to/file.md"}
//! {"type":"directory","path":"/path/to/dir"}
//! {"type":"reopen"}
//! ```

mod client;
mod protocol;
mod queue;
mod server;
pub(crate) mod socket;
mod window_selection;

pub use client::*;
pub use protocol::*;
pub use queue::*;
pub use server::*;
pub use socket::cleanup_socket;

use queue::{drain_events, SHUTDOWN_REQUESTED, SHUTDOWN_SIGNAL, SHUTDOWN_STARTED};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use window_selection::select_target_window;

// ============================================================================
// GCD wake mechanism — wake main thread from IPC background thread
// ============================================================================

/// Wake the main thread to process pending IPC events.
///
/// Uses macOS GCD to dispatch a callback to the main queue, which wakes
/// CFRunLoop even when App Nap has suspended the process. The callback
/// calls `process_main_thread_tasks()` on the main thread.
#[cfg(target_os = "macos")]
fn wake_main_thread() {
    extern "C" {
        // _dispatch_main_q is the GCD main dispatch queue (static symbol in libdispatch).
        // dispatch_get_main_queue() is a C macro that expands to &_dispatch_main_q,
        // so we reference the symbol directly for FFI.
        static _dispatch_main_q: u8;
        fn dispatch_async_f(
            queue: *const u8,
            context: *mut std::ffi::c_void,
            work: extern "C" fn(*mut std::ffi::c_void),
        );
    }

    extern "C" fn ipc_wake_callback(_context: *mut std::ffi::c_void) {
        // Runs on the main thread via GCD — safe to access MAIN_WINDOWS thread_local
        process_main_thread_tasks();
    }

    // SAFETY: _dispatch_main_q is a valid static symbol in libdispatch.
    // dispatch_async_f schedules the callback on the main thread.
    unsafe {
        let main_queue = std::ptr::addr_of!(_dispatch_main_q);
        dispatch_async_f(main_queue, std::ptr::null_mut(), ipc_wake_callback);
    }
}

#[cfg(not(target_os = "macos"))]
fn wake_main_thread() {
    // On non-macOS platforms, rely on custom_event_handler to run
    // process_main_thread_tasks() on the next event loop iteration.
}

// ============================================================================
// Main thread event processing — called from GCD callback or event handler
// ============================================================================

/// Process pending IPC events on the main thread.
///
/// MUST be called on the main thread (accesses MAIN_WINDOWS thread_local).
/// Called from:
/// - GCD wake callback (after IPC thread pushes events)
/// - custom_event_handler (defense in depth)
pub fn process_pending_events() {
    let Some(desktop) = crate::window::get_any_main_window() else {
        // No window available yet; events stay in queue for later processing
        return;
    };

    let events = drain_events();
    for event in events {
        handle_event_on_main_thread(&desktop, event);
    }
}

/// Process all main-thread IPC tasks.
///
/// This drains the open-event queue and handles pending graceful shutdown.
pub fn process_main_thread_tasks() {
    if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
        process_shutdown_request();
        return;
    }

    process_pending_events();
    process_shutdown_request();
}

fn process_shutdown_request() {
    if !SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
        return;
    }
    if SHUTDOWN_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    tracing::info!("Starting graceful shutdown");
    cleanup_socket();
    let closed = crate::window::shutdown_all_windows();
    if closed == 0 {
        let signal = SHUTDOWN_SIGNAL.load(Ordering::SeqCst);
        tracing::warn!(
            signal,
            "No main windows found during shutdown; exiting process directly"
        );
        if signal > 0 {
            std::process::exit(128 + signal);
        }
        std::process::exit(0);
    }
}

/// Handle a single event by creating/showing windows. Runs on main thread.
fn handle_event_on_main_thread(
    desktop: &std::rc::Rc<dioxus::desktop::DesktopService>,
    event: OpenEvent,
) {
    match event {
        OpenEvent::File(path) => {
            tracing::debug!(?path, "Processing file open event");
            open_file_with_behavior(desktop, path);
        }
        OpenEvent::Directory(dir) => {
            tracing::debug!(?dir, "Processing directory open event");
            open_directory_with_behavior(desktop, dir);
        }
        OpenEvent::Reopen => {
            tracing::debug!("Processing reopen event");
            reopen_with_behavior(desktop);
        }
    }
}

fn open_file_with_behavior(desktop: &std::rc::Rc<dioxus::desktop::DesktopService>, path: PathBuf) {
    if let Some(window_id) = select_target_window() {
        if let Some(mut state) = crate::window::main::get_window_state(window_id) {
            state.open_file(&path);
            let _ = crate::window::main::focus_window(window_id);
            return;
        }
    }

    crate::window::create_main_window_sync(
        desktop,
        crate::state::Tab::new(path),
        crate::window::CreateMainWindowConfigParams::default(),
    );
}

fn open_directory_with_behavior(
    desktop: &std::rc::Rc<dioxus::desktop::DesktopService>,
    directory: PathBuf,
) {
    if let Some(window_id) = select_target_window() {
        if let Some(mut state) = crate::window::main::get_window_state(window_id) {
            state.set_root_directory(directory.clone());
            let _ = crate::window::main::focus_window(window_id);
            return;
        }
    }

    let params = crate::window::CreateMainWindowConfigParams {
        directory: Some(directory),
        ..Default::default()
    };
    crate::window::create_main_window_sync(desktop, crate::state::Tab::default(), params);
}

fn reopen_with_behavior(desktop: &std::rc::Rc<dioxus::desktop::DesktopService>) {
    // First try to focus an existing visible window
    if let Some(window_id) = select_target_window() {
        if crate::window::main::focus_window(window_id) {
            return;
        }
    }

    // If no visible windows, try to show and focus a hidden window (e.g., MainApp with WindowHides)
    if crate::window::main::show_and_focus_hidden_window() {
        return;
    }

    // If no windows at all, create a new one
    crate::window::create_main_window_sync(
        desktop,
        crate::state::Tab::default(),
        crate::window::CreateMainWindowConfigParams::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FileOpenBehavior;
    use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
    use indoc::indoc;
    use std::path::{Path, PathBuf};
    use window_selection::{
        choose_open_target, is_window_on_display, OpenTarget, WindowSelectionInput,
    };

    // Re-import protocol types used by tests
    use protocol::IpcMessage;
    use socket::{get_socket_path, is_address_in_use, SOCKET_NAME};

    #[test]
    fn test_ipc_message_file_serialization() {
        let msg = IpcMessage::File {
            path: PathBuf::from("/path/to/file.md"),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"file","path":"/path/to/file.md"}"#);
    }

    #[test]
    fn test_ipc_message_directory_serialization() {
        let msg = IpcMessage::Directory {
            path: PathBuf::from("/path/to/dir"),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"directory","path":"/path/to/dir"}"#);
    }

    #[test]
    fn test_ipc_message_reopen_serialization() {
        let msg = IpcMessage::Reopen;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"reopen"}"#);
    }

    #[test]
    fn test_ipc_message_file_deserialization() {
        let json = r#"{"type":"file","path":"/path/to/file.md"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, IpcMessage::File { path } if path == Path::new("/path/to/file.md")));
    }

    #[test]
    fn test_ipc_message_directory_deserialization() {
        let json = r#"{"type":"directory","path":"/path/to/dir"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, IpcMessage::Directory { path } if path == Path::new("/path/to/dir")));
    }

    #[test]
    fn test_ipc_message_reopen_deserialization() {
        let json = r#"{"type":"reopen"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, IpcMessage::Reopen));
    }

    #[test]
    fn test_ipc_message_into_open_event_file() {
        let msg = IpcMessage::File {
            path: PathBuf::from("/test.md"),
        };
        let event = msg.into_open_event();
        assert!(matches!(event, OpenEvent::File(p) if p == Path::new("/test.md")));
    }

    #[test]
    fn test_ipc_message_into_open_event_directory() {
        let msg = IpcMessage::Directory {
            path: PathBuf::from("/test/dir"),
        };
        let event = msg.into_open_event();
        assert!(matches!(event, OpenEvent::Directory(p) if p == Path::new("/test/dir")));
    }

    #[test]
    fn test_ipc_message_into_open_event_reopen() {
        let msg = IpcMessage::Reopen;
        let event = msg.into_open_event();
        assert!(matches!(event, OpenEvent::Reopen));
    }

    #[test]
    fn test_json_lines_protocol() {
        // Test that multiple messages can be parsed from newline-separated JSON
        let input = indoc! {r#"
            {"type":"file","path":"/file1.md"}
            {"type":"directory","path":"/dir"}
            {"type":"reopen"}
        "#};

        let messages: Vec<IpcMessage> = input
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();

        assert_eq!(messages.len(), 3);
        assert!(
            matches!(&messages[0], IpcMessage::File { path } if path == Path::new("/file1.md"))
        );
        assert!(
            matches!(&messages[1], IpcMessage::Directory { path } if path == Path::new("/dir"))
        );
        assert!(matches!(&messages[2], IpcMessage::Reopen));
    }

    #[test]
    #[cfg(unix)]
    fn test_socket_path_contains_user_id() {
        let path = get_socket_path();

        // Ensure the socket file name is exactly SOCKET_NAME
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("Socket path should have a valid UTF-8 file name");
        assert_eq!(
            file_name, SOCKET_NAME,
            "Socket file name should be '{}', got '{}'",
            SOCKET_NAME, file_name
        );

        // Either XDG_RUNTIME_DIR or /tmp/arto-{uid}/
        let parent = path
            .parent()
            .expect("Socket path should have a parent directory");

        let xdg_runtime_dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
        let parent_matches_xdg = xdg_runtime_dir.as_deref().is_some_and(|xdg| parent == xdg);

        let parent_str = parent.to_string_lossy();
        let parent_matches_tmp = parent_str.starts_with("/tmp/arto-");

        assert!(
            parent_matches_xdg || parent_matches_tmp,
            "Socket directory should be XDG_RUNTIME_DIR ({:?}) or start with '/tmp/arto-'; got {}",
            xdg_runtime_dir,
            parent_str
        );
    }

    #[test]
    fn test_is_address_in_use_for_non_matching_error() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        assert!(!is_address_in_use(&err));
    }

    #[test]
    fn test_event_queue_fifo_ordering() {
        // Drain any leftover events from other tests (global static is shared)
        drain_events();

        push_event(OpenEvent::File(PathBuf::from("/first.md")));
        push_event(OpenEvent::Directory(PathBuf::from("/second")));
        push_event(OpenEvent::Reopen);

        // try_pop_first_event returns FIFO order
        let first = try_pop_first_event();
        assert!(matches!(first, Some(OpenEvent::File(p)) if p == Path::new("/first.md")));

        // drain_events returns remaining in FIFO order
        let remaining = drain_events();
        assert_eq!(remaining.len(), 2);
        assert!(matches!(&remaining[0], OpenEvent::Directory(p) if p == Path::new("/second")));
        assert!(matches!(&remaining[1], OpenEvent::Reopen));

        // Queue is empty after drain
        assert!(try_pop_first_event().is_none());
        assert!(drain_events().is_empty());
    }

    #[test]
    fn test_choose_open_target_new_window_always_creates_new_window() {
        let windows = vec![WindowSelectionInput {
            window_id: 1_u8,
            is_on_current_screen: true,
        }];

        let target = choose_open_target(FileOpenBehavior::NewWindow, &windows, Some(1));
        assert_eq!(target, OpenTarget::NewWindow);
    }

    #[test]
    fn test_choose_open_target_last_focused_uses_visible_window() {
        let windows = vec![
            WindowSelectionInput {
                window_id: 1_u8,
                is_on_current_screen: false,
            },
            WindowSelectionInput {
                window_id: 2_u8,
                is_on_current_screen: true,
            },
        ];

        let target = choose_open_target(FileOpenBehavior::LastFocused, &windows, Some(2));
        assert_eq!(target, OpenTarget::ExistingWindow(2));
    }

    #[test]
    fn test_choose_open_target_last_focused_falls_back_to_new_window() {
        let windows = vec![WindowSelectionInput {
            window_id: 1_u8,
            is_on_current_screen: true,
        }];

        let target = choose_open_target(FileOpenBehavior::LastFocused, &windows, Some(2));
        assert_eq!(target, OpenTarget::NewWindow);
    }

    #[test]
    fn test_choose_open_target_current_screen_prefers_last_focused() {
        let windows = vec![
            WindowSelectionInput {
                window_id: 1_u8,
                is_on_current_screen: true,
            },
            WindowSelectionInput {
                window_id: 2_u8,
                is_on_current_screen: true,
            },
        ];

        let target = choose_open_target(FileOpenBehavior::CurrentScreen, &windows, Some(2));
        assert_eq!(target, OpenTarget::ExistingWindow(2));
    }

    #[test]
    fn test_choose_open_target_current_screen_uses_first_candidate_without_last_focus() {
        let windows = vec![
            WindowSelectionInput {
                window_id: 4_u8,
                is_on_current_screen: true,
            },
            WindowSelectionInput {
                window_id: 5_u8,
                is_on_current_screen: true,
            },
        ];

        let target = choose_open_target(FileOpenBehavior::CurrentScreen, &windows, None);
        assert_eq!(target, OpenTarget::ExistingWindow(4));
    }

    #[test]
    fn test_choose_open_target_current_screen_falls_back_to_new_window() {
        let windows = vec![WindowSelectionInput {
            window_id: 1_u8,
            is_on_current_screen: false,
        }];

        let target = choose_open_target(FileOpenBehavior::CurrentScreen, &windows, Some(1));
        assert_eq!(target, OpenTarget::NewWindow);
    }

    #[test]
    fn test_is_window_on_display_detects_overlap() {
        let display = (LogicalPosition::new(0, 0), LogicalSize::new(1920, 1080));
        let window_position = LogicalPosition::new(1800, 900);
        let window_size = LogicalSize::new(400, 300);

        assert!(is_window_on_display(display, window_position, window_size));
    }

    #[test]
    fn test_is_window_on_display_returns_false_without_overlap() {
        let display = (LogicalPosition::new(0, 0), LogicalSize::new(1920, 1080));
        let window_position = LogicalPosition::new(1921, 0);
        let window_size = LogicalSize::new(400, 300);

        assert!(!is_window_on_display(display, window_position, window_size));
    }
}
