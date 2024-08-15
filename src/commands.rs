use std::path::PathBuf;

#[derive(Clone)]
pub enum FileListCommand {
    EntryScroll(bool),    // true if down scroll
    WindowScroll(bool),   // true if down scroll
    SelectEntry(PathBuf), // Selected entry (doesn't distinguish between dirs/files)
    HintMode,
    FinderMode(bool),     // true if zoxide search
    Yank(bool),           // true if cut
    Paste,

    Exit,
    
    None,
}