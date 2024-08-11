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
    max_entries: u16,
    visible: bool,
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
                        if i == self.selected {
                            Line::from(f.as_str().black().on_white())
                        } else {
                            Line::from(f.as_str())
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
pub struct Window {
    file_list: FileList,
    curr_dir: CurrDirectory,
    controls: Controls,
}

impl Window {
    pub fn new(files: Vec<String>) -> Self {
        let file_list = FileList {
            files,
            scroll: 0,
            selected: 0,
            max_entries: 0,
            visible: true,
        };

        let curr_dir = CurrDirectory {
            curr_directory: "Testing Directory".to_string(),
            visible: true,
        };

        let controls = Controls { visible: true };

        Self {
            file_list,
            curr_dir,
            controls,
        }
    }

    pub fn scroll_list(&mut self, down: bool) {
        if down {
            if self.file_list.scroll
                < self
                    .file_list
                    .files
                    .len()
                    .saturating_sub(self.file_list.max_entries as usize)
            {
                self.file_list.scroll += 1;
            }
        } else {
            self.file_list.scroll = self.file_list.scroll.saturating_sub(1);
        }
    }

    pub fn scroll_entry(&mut self, down: bool) {
        if down {
            if self.file_list.selected < self.file_list.files.len() - 1 {
                self.file_list.selected += 1;
            }
        } else {
            self.file_list.selected = self.file_list.selected.saturating_sub(1);
        }
    }

    pub fn get_curr_entry(&mut self) -> String {
        self.file_list.files[self.file_list.selected].clone()
    }

    pub fn update_files(&mut self, files: Vec<String>) {
        self.file_list.files = files;
    }

    pub fn update_max_entries(&mut self, max_entries: u16) {
        self.file_list.max_entries = max_entries;
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

        if self.file_list.visible {
            self.file_list.render(fl_area, buf);
        }

        if self.curr_dir.visible {
            self.curr_dir.render(cd_area, buf);
        }

        if self.controls.visible {
            self.controls.render(ct_area, buf)
        }
    }
}
