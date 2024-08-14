use bimap::BiHashMap;
use ratatui::{
    prelude::{Buffer, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{block::Title, Block, Paragraph, StatefulWidget, Widget},
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

impl Widget for Finder {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let header_area = Rect::new(area.x, area.y, area.width, 3);
        let content_area = Rect::new(area.x, area.y + 3, area.width, area.height - 1);

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
pub struct Window {
    file_list: FileList,
    curr_dir: CurrDirectory,
    controls: Controls,
    finder: Finder,
}

impl Window {
    pub fn new(files: Vec<String>, starting_dir: String) -> Self {
        let file_list = FileList {
            files,
            scroll: 0,
            selected: 0,
            max_entries: 0,
            hint_mode: false,
            hint_choices: initialize_hints(),
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

        Self {
            file_list,
            curr_dir,
            controls,
            finder,
        }
    }

    pub fn scroll_list(&mut self, down: bool) {
        if down {
            if self.file_list.scroll
                < self
                    .file_list
                    .files
                    .len()
                    .saturating_sub(self.file_list.max_entries)
            {
                self.file_list.scroll += 1;
            }
        } else {
            self.file_list.scroll = self.file_list.scroll.saturating_sub(1);
        }
    }

    pub fn scroll_entry(&mut self, down: bool) {
        if down {
            if self.file_list.scroll + self.file_list.selected >= self.file_list.files.len() - 1 {
                return;
            }

            if self.file_list.selected >= self.file_list.max_entries - 1 {
                self.file_list.scroll +=
                    self.file_list.selected + 1 - (self.file_list.max_entries - 1);
                self.file_list.selected = self.file_list.max_entries - 1;
            } else {
                self.file_list.selected += 1;
            }
        } else {
            if self.file_list.selected == 0 && self.file_list.scroll != 0 {
                self.file_list.scroll -= 1;
            } else {
                self.file_list.selected = self.file_list.selected.saturating_sub(1);
            }
        }
    }

    pub fn hint_mode(&mut self, on: bool) {
        self.file_list.hint_mode = on;
    }

    pub fn valid_hint(&mut self, hint: &String) -> bool {
        self.file_list
            .hint_choices
            .right_values()
            .any(|s| s == hint)
    }

    pub fn jump_hint(&mut self, hint: String) {
        self.file_list.selected = *self
            .file_list
            .hint_choices
            .get_by_right(&hint)
            .expect("Unable to find hint")
    }

    pub fn finder_mode(&mut self, on: bool) {
        if on {
            self.file_list.visible = false;
            self.controls.visible = false;
            self.curr_dir.visible = false;
            self.finder.visible = true;
        } else {
            self.file_list.visible = true;
            self.controls.visible = true;
            self.curr_dir.visible = true;
            self.finder.visible = false;
        }
    }

    pub fn finder_text(&self) -> String {
        self.finder.text.clone()
    }

    pub fn set_finder_text(&mut self, text: String) {
        self.finder.text = text
    }

    pub fn update_finder_files(&mut self, files: Vec<String>) {
        self.finder.files = files
    }

    pub fn scroll_finder(&mut self, down: bool) {
        if down {
            if self.finder.selected < self.finder.max_entries - 1 {
                self.finder.selected += 1;
            }
        } else {
            self.finder.selected = self.finder.selected.saturating_sub(1);
        }
    }

    pub fn finder_selection(&self) -> &'_ String {
        &self.finder.files[self.finder.selected]
    }

    pub fn get_curr_entry(&mut self) -> String {
        self.file_list.files[self.file_list.selected].clone()
    }

    pub fn update_cwd(&mut self, dir: String) {
        self.curr_dir.curr_directory = dir
    }

    pub fn update_files(&mut self, files: Vec<String>) {
        self.file_list.files = files;
    }

    pub fn set_max_entries(&mut self, max_entries: usize) {
        self.file_list.max_entries = max_entries;
    }

    pub fn finder_max_entries(&self) -> usize {
        self.finder.max_entries
    }

    pub fn set_finder_max_entries(&mut self, max_entries: usize) {
        self.finder.max_entries = max_entries
    }
}

impl Widget for Window {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let cd_area = Rect::new(0, 0, area.width, 3);
        let fl_area = Rect::new(0, 3, area.width, area.height - 8);
        let ct_area = Rect::new(0, area.height - 5, area.width, 5);

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

        if self.finder.visible {
            self.finder.render(fd_area, buf);
        }
    }
}
