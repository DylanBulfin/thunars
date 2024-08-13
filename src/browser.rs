use std::{
    fs::DirEntry,
    ops::Add,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    commands::FileListCommand,
    components::{FileList, Window, TOTAL_USED_LINES},
    tui::Tui,
    Result,
};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    widgets::{self, Widget},
};

pub struct Browser {
    window: Window,
    terminal: Tui,
    curr_dir: PathBuf,
    exit: bool,
}

fn fetch_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = std::fs::read_dir(dir)?
        .map(|d| {
            d.expect("Unable to fetch files in directory")
                .file_name()
                .into_string()
                .expect("Unable to process filename")
        })
        .collect::<Vec<_>>();

    files.insert(0, "..".to_string());
    files.insert(0, ".".to_string());

    Ok(files)
}

impl Browser {
    pub fn init(terminal: Tui) -> Result<Browser> {
        let curr_dir = std::env::current_dir()?;
        let files = fetch_files(curr_dir.as_path())?;

        Ok(Self {
            window: Window::new(files, curr_dir.to_string_lossy().to_string()),
            terminal,
            curr_dir,
            exit: false,
        })
    }

    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            f.render_widget(self.window.clone(), f.area());
            self.window
                .update_max_entries((f.area().height - TOTAL_USED_LINES) as usize);
        })?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    event::Event::Key(ke) => {
                        let command = self.hande_key_event(ke);
                        self.execute_command(command)?
                    }
                    _ => (),
                }
            }

            self.draw()?;

            if self.exit {
                return Ok(());
            }
        }
    }

    fn change_directory(&mut self, new_dir: PathBuf) -> Result<()> {
        self.curr_dir = new_dir.canonicalize().expect("Trying to cd to non-existent directory");
        
        let mut sorted_files =fetch_files(self.curr_dir.as_path())?;
        sorted_files.sort();

        self.window
            .update_files(sorted_files);
        self.window
            .update_cwd(self.curr_dir.to_string_lossy().to_string());

        Ok(())
    }

    fn hande_key_event(&mut self, ke: KeyEvent) -> FileListCommand {
        if ke.kind == KeyEventKind::Press {
            match ke.code {
                KeyCode::Char(c) => {
                    if c == 'q' {
                        FileListCommand::Exit
                    } else if c == 'd' {
                        FileListCommand::WindowScroll(true)
                    } else if c == 'u' {
                        FileListCommand::WindowScroll(false)
                    } else if c == 'n' {
                        FileListCommand::EntryScroll(true)
                    } else if c == 'e' {
                        FileListCommand::EntryScroll(false)
                    } else if c == 'f' {
                        FileListCommand::HintMode
                    } else {
                        FileListCommand::None
                    }
                }
                KeyCode::Enter => FileListCommand::SelectEntry(self.window.get_curr_entry()),
                _ => FileListCommand::None,
            }
        } else {
            FileListCommand::None
        }
    }

    fn hint_mode(&mut self) -> Result<()> {
        self.window.hint_mode(true);
        let mut hint = String::new();

        loop {
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    event::Event::Key(ke) => {
                        if ke.kind == KeyEventKind::Press {
                            match ke.code {
                                KeyCode::Esc => break,
                                KeyCode::Char(c) => {
                                    hint.push(c);
                                    if self.window.valid_hint(&hint) {
                                        break;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }

            self.draw()?;
        }

        if self.window.valid_hint(&hint) {
            self.window.jump_hint(hint)
        }
        
        self.window.hint_mode(false);

        Ok(())
    }

    fn execute_command(&mut self, comm: FileListCommand) -> Result<()> {
        match comm {
            FileListCommand::EntryScroll(d) => self.window.scroll_entry(d),
            FileListCommand::WindowScroll(d) => self.window.scroll_list(d),
            FileListCommand::SelectEntry(e) => {
                let mut new_path = self.curr_dir.clone();
                new_path.push(e);
                if new_path.is_dir() {
                    self.change_directory(new_path)?;
                } else if new_path.is_file() {
                    unimplemented!()
                }
            }
            FileListCommand::Exit => self.exit = true,
            FileListCommand::HintMode => self.hint_mode()?,
            _ => (),
        };

        Ok(())
    }

    // fn render(&mut self) -> Result<()> {
    //     self.terminal.draw(|f| {
    //         let mut buf = f.buffer_mut();
    //         for segment in self.segments.iter() {
    //             segment.component.render(segment.area, buf);
    //         }
    //     })?;

    //     unimplemented!()
    // }
}
