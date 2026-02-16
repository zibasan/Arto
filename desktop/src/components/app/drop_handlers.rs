use dioxus::html::HasFileData;
use dioxus::prelude::*;

use crate::state::AppState;

/// Handle dropped files/directories - opens markdown files or sets directory as root
pub(super) async fn handle_dropped_files(evt: Event<DragData>, mut state: AppState) {
    let files = evt.files();
    if files.is_empty() {
        return;
    }

    for file_data in files {
        let path = file_data.path();

        // Resolve symlinks and canonicalize the path to handle Finder sidebar items
        let resolved_path = match std::fs::canonicalize(&path) {
            Ok(p) => {
                tracing::info!("Resolved path: {:?} -> {:?}", path, p);
                p
            }
            Err(e) => {
                tracing::warn!("Failed to canonicalize path {:?}: {}", path, e);
                path.clone()
            }
        };

        tracing::info!(
            "Processing dropped path: {:?}, is_dir: {}",
            resolved_path,
            resolved_path.is_dir()
        );

        if resolved_path.is_dir() {
            // If it's a directory, set it as root and show the sidebar
            tracing::info!("Setting dropped directory as root: {:?}", resolved_path);
            state.set_root_directory(resolved_path);
            // Show the sidebar if it's hidden so users can see the directory tree
            if !state.sidebar.read().open {
                state.toggle_sidebar();
            }
        } else {
            // Open any file (not just markdown)
            tracing::info!("Opening dropped file: {:?}", resolved_path);
            state.open_file(resolved_path);
        }
    }
}
