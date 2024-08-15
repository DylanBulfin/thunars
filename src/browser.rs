use std::{
    env::{current_dir, set_current_dir},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    commands::FileListCommand,
    components::{Window, BLOCK_LINES, CURR_DIR_LINES, TOTAL_USED_LINES},
    tui::Tui,
    Result,
};
use ignore::Walk;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

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
                .file_list
                .set_max_entries((f.area().height - TOTAL_USED_LINES) as usize);
            self.window
                .finder
                .set_finder_max_entries((f.area().height - CURR_DIR_LINES - BLOCK_LINES) as usize);
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
        self.curr_dir = new_dir
            .canonicalize()
            .expect("Trying to cd to non-existent directory");

        set_current_dir(self.curr_dir.as_path()).expect("Unable to change working directory");

        let mut sorted_files = fetch_files(self.curr_dir.as_path())?;
        sorted_files.sort();

        self.window.file_list.update_files(sorted_files);
        self.window
            .curr_dir
            .update_cwd(self.curr_dir.to_string_lossy().to_string());

        Ok(())
    }

    fn open_file(&mut self, file: PathBuf) -> Result<()> {
        let program = "code";

        Command::new(program).arg(file).spawn()?;

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
                    } else if c == '/' {
                        FileListCommand::FinderMode(false)
                    } else if c == 'z' {
                        FileListCommand::FinderMode(true)
                    } else {
                        FileListCommand::None
                    }
                }
                KeyCode::Enter => {
                    FileListCommand::SelectEntry(self.window.file_list.get_curr_entry().into())
                }
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
                    Event::Key(ke) => {
                        if ke.kind == KeyEventKind::Press {
                            match ke.code {
                                KeyCode::Esc => break,
                                KeyCode::Char(c) => {
                                    hint.push(c);
                                    if self.window.file_list.valid_hint(&hint) {
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

        if self.window.file_list.valid_hint(&hint) {
            self.window.file_list.jump_hint(hint)
        }

        self.window.hint_mode(false);

        Ok(())
    }

    fn open_entry(&mut self, entry: PathBuf) -> Result<()> {
        if entry.is_dir() {
            self.change_directory(entry)?;
        } else {
            self.open_file(entry)?;
        }

        Ok(())
    }

    fn find(&self, text: String, zoxide: bool) -> Result<Vec<String>> {
        if zoxide {
            let command = Command::new("zoxide")
                .arg("query")
                .arg("--list")
                .arg(text)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .spawn()?;

            Ok(String::from_utf8(command.wait_with_output()?.stdout)
                .expect("Couldn't read zoxide output")
                .lines()
                .take(self.window.finder.finder_max_entries())
                .map(Into::into)
                .collect())
        } else {
            let path = current_dir().expect("Unable to access current dir");

            let items = Walk::new(PathBuf::from(&path))
                .map(|r| {
                    r.expect("Failed to process file")
                        .into_path()
                        .strip_prefix(&path)
                        .expect("Found file not in curr directory")
                        .to_string_lossy()
                        .to_string()
                })
                .filter(|i| i.len() > 0) // Walk has directory itself as member, filter that out
                .collect::<Vec<_>>();

            if text.len() == 0 {
                return Ok(items);
            }

            let mut command = Command::new("fzf")
                .arg("-f")
                .arg(&text)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            for i in items {
                let stdin = command
                    .stdin
                    .as_mut()
                    .expect("Unable to access stdin for fzf");
                stdin.write_fmt(format_args!("{}\n", i))?;
                stdin.flush()?;
            }

            Ok(String::from_utf8(command.wait_with_output()?.stdout)
                .expect("Unable to read fzf output")
                .lines()
                .take(self.window.finder.finder_max_entries())
                .map(Into::into)
                .collect())
        }
    }

    fn finder_mode(&mut self, zoxide: bool) -> Result<()> {
        self.window.finder_mode(true);
        self.window
            .finder
            .update_finder_files(self.find(String::new(), zoxide)?);

        loop {
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(ke) => {
                        if ke.kind == KeyEventKind::Press {
                            match ke.code {
                                KeyCode::Esc => break,
                                KeyCode::Char(c) => {
                                    let mut text = self.window.finder.finder_text();
                                    text.push(c);
                                    self.window
                                        .finder
                                        .update_finder_files(self.find(text.clone(), zoxide)?);
                                    self.window.finder.set_finder_text(text);
                                }
                                KeyCode::Backspace => {
                                    let mut text = self.window.finder.finder_text();
                                    if text.len() > 0 {
                                        text.remove(text.len() - 1);
                                        self.window
                                            .finder
                                            .update_finder_files(self.find(text.clone(), zoxide)?);
                                        self.window.finder.set_finder_text(text);
                                    }
                                }
                                KeyCode::Enter => {
                                    self.open_entry(self.window.finder.finder_selection().into())?;
                                    break;
                                }
                                KeyCode::Down | KeyCode::Tab => {
                                    self.window.finder.scroll_finder(true)
                                }
                                KeyCode::Up | KeyCode::BackTab => {
                                    self.window.finder.scroll_finder(false)
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }

            self.draw()?
        }

        self.window.finder_mode(false);
        self.window.finder.update_finder_files(Vec::new());
        self.window.finder.set_finder_text(String::new());
        self.window.finder.finder_reset();

        Ok(())
    }

    fn execute_command(&mut self, comm: FileListCommand) -> Result<()> {
        match comm {
            FileListCommand::EntryScroll(d) => self.window.file_list.scroll_entry(d),
            FileListCommand::WindowScroll(d) => self.window.file_list.scroll_list(d),
            FileListCommand::SelectEntry(p) => self.open_entry(p)?,
            FileListCommand::Exit => self.exit = true,
            FileListCommand::HintMode => self.hint_mode()?,
            FileListCommand::FinderMode(z) => self.finder_mode(z)?,
            FileListCommand::None => (),
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
