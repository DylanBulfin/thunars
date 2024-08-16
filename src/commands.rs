use std::path::PathBuf;

#[derive(Clone)]
pub enum FileListCommand {
    EntryScroll(bool),    // true if down scroll
    WindowScroll(bool),   // true if down scroll
    SelectEntry(PathBuf), // Selected entry (doesn't distinguish between dirs/files)
    HintMode,
    FinderMode(bool), // true if zoxide search
    RenameMode,
    Yank(bool),       // true if cut
    Paste,

    Exit,

    None,
}

impl FileListCommand {
    pub fn refresh_preview(&self) -> bool {
        match self {
            FileListCommand::Exit | FileListCommand::None => false,
            _ => true,
        }
    }
}
