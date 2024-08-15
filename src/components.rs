use std::path::PathBuf;

use bimap::BiHashMap;
use ratatui::{
    prelude::{Buffer, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{block::Title, Block, Paragraph, Widget},
};

pub const BLOCK_LINES: u16 = 2;

pub const CURR_DIR_LINES: u16 = 1;
pub const CURR_DIR_HEIGHT: u16 = BLOCK_LINES + CURR_DIR_LINES;

pub const CONTROLS_LINES: u16 = 3;
pub const CONTROLS_HEIGHT: u16 = BLOCK_LINES + CONTROLS_LINES;

pub const TOTAL_USED_LINES: u16 = BLOCK_LINES + CURR_DIR_HEIGHT + CONTROLS_HEIGHT;

#[derive(Clone)]
pub struct FileList {
    files: Vec<String>,
    scroll: usize,
    selected: usize,
    max_entries: usize,
    hint_mode: bool,
    hint_choices: BiHashMap<usize, String>,
    visible: bool,
}

impl FileList {
    fn initialize_hints() -> BiHashMap<usize, String> {
        let one_letter = ["p", "l", "f", "u", "w", "y", "q", ";"].map(|s| s.to_string());

        let first_options = ["t", "n", "s", "e", "r", "i", "a", "o"];
        let second_options = [
            "t", "n", "s", "e", "r", "i", "a", "o", "p", "l", "f", "u", "w", "y", "q", ";",
        ];
        let two_letters = first_options
            .into_iter()
            .flat_map(|c1| second_options.into_iter().map(|c2| c1.to_string() + c2))
            .collect::<Vec<_>>();

        let mut hints = BiHashMap::new();
        for (i, o) in one_letter.iter().enumerate() {
            hints.insert(i, o.to_string());
        }
        for (i, t) in two_letters.into_iter().enumerate() {
            hints.insert(i + one_letter.len(), t);
        }

        hints
    }

    pub fn update_files(&mut self, files: Vec<String>) {
        self.files = files;
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn set_max_entries(&mut self, max_entries: usize) {
        self.max_entries = max_entries;
    }

    pub fn scroll_list(&mut self, down: bool) {
        if down {
            if self.scroll < self.files.len().saturating_sub(self.max_entries) {
                self.scroll += 1;
            }
        } else {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    pub fn scroll_entry(&mut self, down: bool) {
        if down {
            if self.scroll + self.selected >= self.files.len() - 1 {
                return;
            }

            if self.selected >= self.max_entries - 1 {
                self.scroll += self.selected + 1 - (self.max_entries - 1);
                self.selected = self.max_entries - 1;
            } else {
                self.selected += 1;
            }
        } else {
            if self.selected == 0 && self.scroll != 0 {
                self.scroll -= 1;
            } else {
                self.selected = self.selected.saturating_sub(1);
            }
        }
    }

    fn hint_mode(&mut self, on: bool) {
        self.hint_mode = on;
    }

    pub fn valid_hint(&mut self, hint: &String) -> bool {
        self.hint_choices.right_values().any(|s| s == hint)
    }

    pub fn jump_hint(&mut self, hint: String) {
        self.selected = *self
            .hint_choices
            .get_by_right(&hint)
            .expect("Unable to find hint")
    }

    pub fn curr_entry(&self) -> String {
        self.files[self.selected].clone()
    }
}

impl Widget for FileList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from("Files");
        let max_index = self.files.len().min(self.scroll + area.height as usize);
        let text = if max_index > self.scroll {
            Text::from(
                self.files[self.scroll..max_index]
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        if self.hint_mode {
                            let hint = self
                                .hint_choices
                                .get_by_left(&i)
                                .expect("Unable to process hint")
                                .clone();
                            if hint.len() == 1 {
                                Line::from(vec![
                                    hint.green().on_black(),
                                    "  ".on_black(),
                                    f.clone().gray().on_black(),
                                ])
                            } else {
                                Line::from(vec![
                                    hint.blue().on_black(),
                                    " ".on_black(),
                                    f.clone().gray().on_black(),
                                ])
                            }
                        } else {
                            if i == self.selected {
                                Line::from(vec!["   ".on_black(), f.clone().black().on_white()])
                            } else {
                                Line::from(vec!["   ".on_black(), f.clone().white().on_black()])
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            Text::from("")
        };

        let block = Block::bordered().title(title);

        Paragraph::new(text).block(block).render(area, buf);
    }
}

#[derive(Clone)]
pub struct CurrDirectory {
    curr_directory: String,
    visible: bool,
}

impl CurrDirectory {
    pub fn update_cwd(&mut self, dir: String) {
        self.curr_directory = dir
    }
}

impl Widget for CurrDirectory {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text = Text::from(self.curr_directory.green()).bold();
        let title = Title::from("CD");
        let block = Block::bordered().title(title);

        Paragraph::new(text).block(block).render(area, buf);
    }
}

#[derive(Clone)]
pub struct Controls {
    visible: bool,
}

impl Widget for Controls {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text = Text::from(vec![Line::from("a".red()), "c".into(), "b".bold().into()]);
        let title = Title::from("Controls");
        let block = Block::bordered().title(title);

        Paragraph::new(text).block(block).render(area, buf)
    }
}

#[derive(Clone)]
pub struct Finder {
    visible: bool,
    text: String,
    selected: usize,
    max_entries: usize,
    files: Vec<String>,
}

impl Finder {
    pub fn reset(&mut self) {
        self.selected = 0;
        self.files = Vec::new();
        self.text = String::new();
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text
    }

    pub fn update_files(&mut self, files: Vec<String>) {
        self.files = files;
        self.files.truncate(self.max_entries);

        if self.selected != 0 && self.selected >= self.files.len() {
            self.selected = self.files.len().saturating_sub(1);
        }
    }

    pub fn scroll(&mut self, down: bool) {
        if down {
            if self.selected < self.max_entries - 1 {
                self.selected += 1;
            }
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    pub fn selection(&self) -> &'_ String {
        &self.files[self.selected]
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn set_max_entries(&mut self, max_entries: usize) {
        self.max_entries = max_entries
    }
}

impl Widget for Finder {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let header_area = Rect::new(area.x, area.y, area.width, 3);
        let content_area = Rect::new(area.x, area.y + 3, area.width, area.height - 3);

        let header_text = Text::from(self.text);
        let header_block = Block::bordered();

        let text = Text::from(
            self.files
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    if i == self.selected {
                        Line::from(s.as_str().black().on_white())
                    } else {
                        Line::from(s.as_str())
                    }
                })
                .collect::<Vec<_>>(),
        );
        let block = Block::bordered();

        Paragraph::new(header_text)
            .block(header_block)
            .render(header_area, buf);
        Paragraph::new(text).block(block).render(content_area, buf);
    }
}

#[derive(Clone)]
pub struct ClipboardEntry {
    file: PathBuf,
    cut: bool,
}

impl ClipboardEntry {
    pub fn new(file: PathBuf, cut: bool) -> Self {
        Self { file, cut }
    }

    pub fn fname(&self) -> String {
        self.file.to_string_lossy().to_string()
    }

    pub fn format(&self, width: usize) -> String {
        let fname = self.fname();
        let fname_len = fname.char_indices().count();
        let max_width = width - 2 - BLOCK_LINES as usize; // 2 to leave room for mode marker

        let mode = if self.cut { "X" } else { "Y" };

        if fname_len > max_width {
            let mut chars = fname.chars();
            for _ in 0..fname_len - max_width + 3 {
                chars.next();
            }
            format!(
                "...{} {}",
                chars.collect::<String>(),
                mode
            )
        } else {
            format!("{:<max_width$} {}", fname, mode)
        }
    }
    
    pub fn file(&self) -> &PathBuf {
        &self.file
    }
    
    pub fn cut(&self) -> bool {
        self.cut
    }
}

#[derive(Clone)]
pub struct Clipboard {
    visible: bool,
    files: Vec<ClipboardEntry>,
    max_entries: usize,
}

impl Clipboard {
    pub fn push(&mut self, file: ClipboardEntry) {
        self.files.push(file);
    }

    pub fn append(&mut self, mut files: Vec<ClipboardEntry>) {
        self.files.append(&mut files);
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn get_files(&self) -> &'_ Vec<ClipboardEntry> {
        &self.files
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn set_max_entries(&mut self, n: usize) {
        self.max_entries = n;
    }
}

impl Widget for Clipboard {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from("Clipboard");
        let text = Text::from(
            self.files
                .iter()
                .map(|ce| Line::from(ce.format(area.width as usize)))
                .collect::<Vec<_>>(),
        );
        let block = Block::bordered().title(title);

        Paragraph::new(text).block(block).render(area, buf);
    }
}

#[derive(Clone)]
pub struct Window {
    pub(crate) file_list: FileList,
    pub(crate) curr_dir: CurrDirectory,
    pub(crate) controls: Controls,
    pub(crate) finder: Finder,
    pub(crate) clipboard: Clipboard,
}

impl Window {
    pub fn new(files: Vec<String>, starting_dir: String) -> Self {
        let file_list = FileList {
            files,
            scroll: 0,
            selected: 0,
            max_entries: 0,
            hint_mode: false,
            hint_choices: FileList::initialize_hints(),
            visible: true,
        };

        let curr_dir = CurrDirectory {
            curr_directory: starting_dir,
            visible: true,
        };

        let controls = Controls { visible: true };

        let finder = Finder {
            visible: false,
            selected: 0,
            max_entries: 0,
            text: String::new(),
            files: Vec::new(),
        };

        let clipboard = Clipboard {
            visible: true,
            files: Vec::new(),
            max_entries: 0,
        };

        Self {
            file_list,
            curr_dir,
            controls,
            finder,
            clipboard,
        }
    }

    pub fn hint_mode(&mut self, on: bool) {
        self.file_list.hint_mode(on);
    }

    pub fn finder_mode(&mut self, on: bool) {
        if on {
            self.file_list.visible = false;
            self.controls.visible = false;
            self.curr_dir.visible = false;
            self.clipboard.visible = false;
            self.finder.visible = true;
        } else {
            self.file_list.visible = true;
            self.controls.visible = true;
            self.curr_dir.visible = true;
            self.clipboard.visible = true;
            self.finder.visible = false;
        }
    }
}

impl Widget for Window {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let cd_area = Rect::new(0, 0, 3 * area.width / 4, 3);
        let fl_area = Rect::new(0, 3, 3 * area.width / 4, area.height - 8);
        let ct_area = Rect::new(0, area.height - 5, 3 * area.width / 4, 5);
        let cb_area = Rect::new(
            3 * area.width / 4,
            area.height / 2,
            area.width - 3 * area.width / 4,
            area.height - area.height / 2,
        );

        let fd_area = Rect::new(area.width / 8, 0, 3 * area.width / 4, area.height);

        if self.file_list.visible {
            self.file_list.render(fl_area, buf);
        }

        if self.curr_dir.visible {
            self.curr_dir.render(cd_area, buf);
        }

        if self.controls.visible {
            self.controls.render(ct_area, buf);
        }

        if self.clipboard.visible {
            self.clipboard.render(cb_area, buf);
        }

        if self.finder.visible {
            self.finder.render(fd_area, buf);
        }
    }
}
