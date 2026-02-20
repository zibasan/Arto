use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliOpenMode {
    LastFocused,
    CurrentScreen,
    NewWindow,
}

impl CliOpenMode {
    pub(crate) fn to_file_open_behavior(self) -> crate::config::FileOpenBehavior {
        match self {
            Self::LastFocused => crate::config::FileOpenBehavior::LastFocused,
            Self::CurrentScreen => crate::config::FileOpenBehavior::CurrentScreen,
            Self::NewWindow => crate::config::FileOpenBehavior::NewWindow,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CliInvocation {
    pub paths: Vec<PathBuf>,
    pub directory: Option<PathBuf>,
    pub open_mode: CliOpenMode,
}
