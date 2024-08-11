#[derive(Clone)]
pub enum FileListCommand {
    EntryScroll(bool),   // true if down scroll
    WindowScroll(bool),  // true if down scroll
    SelectEntry(String), // Selected entry (doesn't distinguish between dirs/files)
    Exit,
    
    None,
}