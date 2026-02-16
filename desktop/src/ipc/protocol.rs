use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Open event types for distinguishing files, directories, and reopen events.
///
/// Used to communicate between OS event handlers (main.rs custom_event_handler),
/// IPC server (secondary instance), and the initial window setup (MainApp).
#[derive(Debug, Clone)]
pub enum OpenEvent {
    /// File opened from Finder/CLI
    File(PathBuf),
    /// Directory opened from Finder/CLI (should set sidebar root)
    Directory(PathBuf),
    /// App icon clicked (reopen event)
    Reopen,
}

/// IPC message types sent between instances as JSON Lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum IpcMessage {
    /// Open a file
    File { path: PathBuf },
    /// Open a directory (set as sidebar root)
    Directory { path: PathBuf },
    /// Reopen/activate the application (no arguments provided)
    Reopen,
}

impl IpcMessage {
    /// Convert to OpenEvent for internal use.
    pub(super) fn into_open_event(self) -> OpenEvent {
        match self {
            IpcMessage::File { path } => OpenEvent::File(path),
            IpcMessage::Directory { path } => OpenEvent::Directory(path),
            IpcMessage::Reopen => OpenEvent::Reopen,
        }
    }

    /// Validate and categorize a path as File or Directory message.
    ///
    /// This helper canonicalizes the path (resolving symlinks), checks if it's a file
    /// or directory, and returns the appropriate IpcMessage variant.
    ///
    /// # Returns
    ///
    /// - `Some(IpcMessage::File)` if the path is a file
    /// - `Some(IpcMessage::Directory)` if the path is a directory
    /// - `None` if the path is invalid (neither file nor directory)
    pub(super) fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if canonical.is_dir() {
            Some(IpcMessage::Directory { path: canonical })
        } else if canonical.is_file() {
            Some(IpcMessage::File { path: canonical })
        } else {
            tracing::warn!(?path, "Skipping invalid path (not a file or directory)");
            None
        }
    }
}

/// Validate and categorize a path as an OpenEvent.
///
/// This helper canonicalizes the path (resolving symlinks), checks if it's a file
/// or directory, and returns the appropriate OpenEvent variant.
///
/// # Returns
///
/// - `Some(OpenEvent::File)` if the path is a file
/// - `Some(OpenEvent::Directory)` if the path is a directory
/// - `None` if the path is invalid (neither file nor directory)
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// # use arto::ipc::{OpenEvent, validate_path};
///
/// let event: Option<OpenEvent> = validate_path(Path::new("/path/to/file.md"));
/// ```
pub fn validate_path(path: impl AsRef<Path>) -> Option<OpenEvent> {
    IpcMessage::from_path(path).map(|msg| msg.into_open_event())
}

/// Result of trying to send paths to an existing instance.
pub enum SendResult {
    /// Successfully sent paths to existing instance - caller should exit
    Sent,
    /// No existing instance found - caller should become primary
    NoExistingInstance,
}
