use std::{collections::HashMap, fs};

use crate::Result;

use homedir::my_home;
use ratatui::crossterm::event::KeyCode;
use toml::{Table, Value};

#[derive(Clone, Copy, Debug)]
pub enum OmnibarType {
    Rename,
    Touch,
    Mkdir,
}

#[derive(Clone, Copy, Debug)]
pub enum FileListCommand {
    EntryScroll(bool), // true if down scroll
    SelectEntry,       // Selected entry (doesn't distinguish between dirs/files)
    HintMode,
    FinderMode(bool), // true if zoxide search
    OmnibarMode(OmnibarType),
    Yank(bool), // true if cut
    Paste,
    Delete(bool), // true if force (meaning it can delete directories)

    Exit,
    ExitHint,

    None,
}

impl FileListCommand {
    pub fn should_refresh_preview(&self) -> bool {
        !matches!(self, FileListCommand::Exit | FileListCommand::None)
    }
}

impl From<&str> for FileListCommand {
    fn from(value: &str) -> Self {
        match value {
            "scroll_down" => FileListCommand::EntryScroll(true),
            "scroll_up" => FileListCommand::EntryScroll(false),
            "select_entry" => FileListCommand::SelectEntry,
            "hint_mode" => FileListCommand::HintMode,
            "finder_fzf" => FileListCommand::FinderMode(false),
            "finder_zoxide" => FileListCommand::FinderMode(true),
            "rename" => FileListCommand::OmnibarMode(OmnibarType::Rename),
            "touch" => FileListCommand::OmnibarMode(OmnibarType::Touch),
            "mkdir" => FileListCommand::OmnibarMode(OmnibarType::Mkdir),
            "yank" => FileListCommand::Yank(false),
            "cut" => FileListCommand::Yank(true),
            "paste" => FileListCommand::Paste,
            "exit" => FileListCommand::Exit,
            "exit_hint" => FileListCommand::ExitHint,
            _ => FileListCommand::None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum FinderCommand {
    Write(char),
    Backspace,
    SelectEntry,
    EntryScroll(bool),

    Exit,

    None,
}

impl From<&str> for FinderCommand {
    fn from(value: &str) -> Self {
        match value {
            "backspace" => FinderCommand::Backspace,
            "select_entry" => FinderCommand::SelectEntry,
            "scroll_down" => FinderCommand::EntryScroll(true),
            "scroll_up" => FinderCommand::EntryScroll(false),
            "exit" => FinderCommand::Exit,
            _ => FinderCommand::None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OmnibarCommand {
    Write(char),
    Backspace,

    Submit,

    Exit,

    None,
}

impl From<&str> for OmnibarCommand {
    fn from(value: &str) -> Self {
        match value {
            "backspace" => OmnibarCommand::Backspace,
            "submit" => OmnibarCommand::Submit,
            "exit" => OmnibarCommand::Exit,
            _ => OmnibarCommand::None,
        }
    }
}

pub struct Config {
    file_list_bindings: HashMap<KeyCode, FileListCommand>,
    finder_bindings: HashMap<KeyCode, FinderCommand>,
    omnibar_bindings: HashMap<KeyCode, OmnibarCommand>,
}

impl Config {
    pub fn init() -> Result<Self> {
        let mut conf = my_home()?.expect("Unable to find home directory");
        conf.push(".config/thunars/config.toml");

        let default_config = include_str!("../config/default.toml");

        let user_table = match fs::read_to_string(conf) {
            Ok(s) => s.parse().expect("Ill-formatted toml"),
            Err(_) => Table::new(),
        };

        let default_table = default_config
            .parse()
            .expect("Default config formatted incorrectly");

        Ok(Self {
            file_list_bindings: Self::init_file_list(&user_table, &default_table)?,
            finder_bindings: Self::init_finder(&user_table, &default_table)?,
            omnibar_bindings: Self::init_omnibar(&user_table, &default_table)?,
        })
    }

    fn init_file_list(
        user_table: &Table,
        default_table: &Table,
    ) -> Result<HashMap<KeyCode, FileListCommand>> {
        let user_bindings = if let Some(Value::Table(t)) = user_table.get("filelist") {
            t
        } else {
           &Table::new()
        };

        let default_bindings = default_table
            .get("filelist")
            .expect("Unable to parse default config")
            .as_table()
            .expect("filelist section in default config is corrupted");

        let mut map = HashMap::new();

        let keys = [
            "scroll_down",
            "scroll_up",
            "select_entry",
            "hint_mode",
            "finder_fzf",
            "finder_zoxide",
            "rename",
            "touch",
            "mkdir",
            "yank",
            "cut",
            "paste",
            "exit",
            "exit_hint"
        ];

        for k in keys {
            let str = if let Some(Value::String(s)) = user_bindings.get(k) {
                s
            } else {
                default_bindings
                    .get(k)
                    .expect("Unable to parse default config")
                    .as_str()
                    .expect("Unable to parse default config")
            };

            let code = keycode_from_str(str);

            map.insert(code, k.into());
        }
        
        Ok(map)
    }

    fn init_finder(
        user_table: &Table,
        default_table: &Table,
    ) -> Result<HashMap<KeyCode, FinderCommand>> {
        let user_bindings = if let Some(Value::Table(t)) = user_table.get("finder") {
            t
        } else {
            &Table::new()
        };

        let default_bindings = default_table
            .get("finder")
            .expect("Unable to parse default config")
            .as_table()
            .expect("filelist section in default config is corrupted");

        let mut map = HashMap::new();

        let keys = [
            "backspace",
            "select_entry",
            "scroll_down",
            "scroll_up",
            "exit",
        ];

        for k in keys {
            let str = if let Some(Value::String(s)) = user_bindings.get(k) {
                if s.len() <= 1 {
                    panic!("Can't assign char key in finder mode")
                } else {
                    s
                }
            } else {
                default_bindings
                    .get(k)
                    .expect("Unable to parse default config")
                    .as_str()
                    .expect("Unable to parse default config")
            };

            let code = keycode_from_str(str);

            map.insert(code, k.into());
        }

        Ok(map)
    }

    fn init_omnibar(
        user_table: &Table,
        default_table: &Table,
    ) -> Result<HashMap<KeyCode, OmnibarCommand>> {
        let user_bindings = if let Some(Value::Table(t)) = user_table.get("omnibar") {
            t
        } else {
            &Table::new()
        };

        let default_bindings = default_table
            .get("omnibar")
            .expect("Unable to parse default config")
            .as_table()
            .expect("filelist section in default config is corrupted");

        let mut map = HashMap::new();

        let keys = ["backspace", "submit", "exit"];

        for k in keys {
            let str = if let Some(Value::String(s)) = user_bindings.get(k) {
                if s.len() <= 1 {
                    panic!("Can't assign char key in omnibar mode")
                } else {
                    s
                }
            } else {
                default_bindings
                    .get(k)
                    .expect("Unable to parse default config")
                    .as_str()
                    .expect("Unable to parse default config")
            };

            let code = keycode_from_str(str);

            map.insert(code, k.into());
        }

        Ok(map)
    }

    pub fn get_filelist_command(&self, code: KeyCode) -> Option<FileListCommand> {
        self.file_list_bindings.get(&code).copied()
    }

    pub fn get_finder_command(&self, code: KeyCode) -> Option<FinderCommand> {
        if let KeyCode::Char(c) = code {
            Some(FinderCommand::Write(c))
        } else {
            self.finder_bindings.get(&code).copied()
        }
    }

    pub fn get_omnibar_command(&self, code: KeyCode) -> Option<OmnibarCommand> {
        if let KeyCode::Char(c) = code {
            Some(OmnibarCommand::Write(c))
        } else {
            self.omnibar_bindings.get(&code).copied()
        }
    }
}

pub fn keycode_from_str(s: &str) -> KeyCode {
    if s.len() == 1 {
        KeyCode::Char(s.chars().next().expect("Unable to parse char key binding"))
    } else {
        match s {
            "enter" => KeyCode::Enter,
            "backspace" => KeyCode::Backspace,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "esc" => KeyCode::Esc,
            _ => KeyCode::Null,
        }
    }
}
