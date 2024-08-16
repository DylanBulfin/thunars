use std::{
    env::{current_dir, set_current_dir},
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    commands::{FileListCommand, OmnibarMode},
    components::{ClipboardEntry, File, Window, BLOCK_LINES, CURR_DIR_LINES, TOTAL_USED_LINES},
    tui::Tui,
    Result,
};
use ignore::Walk;
use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    style::Color,
};

pub struct Browser {
    window: Window,
    terminal: Tui,
    curr_dir: PathBuf,
    exit: bool,
}

fn fetch_files(dir: &Path) -> Result<Vec<File>> {
    let paths = std::fs::read_dir(dir)?
        .map(|d| d.expect("Unable to fetch files in directory").path())
        .collect::<Vec<_>>();

    let (mut dirs, mut files): (Vec<_>, Vec<_>) = paths.into_iter().partition(|p| p.is_dir());
    dirs.sort();
    files.sort();

    let mut entries: Vec<_> = dirs
        .into_iter()
        .map(|p| {
            p.strip_prefix(dir)
                .expect("Unable to parse dir")
                .to_string_lossy()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .map(|d| File::new(d, Color::DarkYellow))
        .collect();

    entries.append(
        &mut files
            .into_iter()
            .map(|p| {
                p.strip_prefix(dir)
                    .expect("Unable to parse file")
                    .to_string_lossy()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .map(|f| File::new(f, Color::Cyan))
            .collect(),
    );

    entries.insert(0, File::new(".".to_string(), Color::White));
    entries.insert(0, File::new("..".to_string(), Color::White));

    Ok(entries)
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
                .set_max_entries((f.area().height - CURR_DIR_LINES - BLOCK_LINES) as usize);
            self.window
                .preview
                .set_max_lines((f.area().height - f.area().height / 3 - BLOCK_LINES) as usize);
            self.window
                .clipboard
                .set_max_entries((f.area().height / 3 - BLOCK_LINES) as usize);
        })?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(16))? {
                if let event::Event::Key(ke) = event::read()? {
                    let command = self.hande_key_event(ke);
                    self.execute_command(command)?
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

        let sorted_files = fetch_files(self.curr_dir.as_path())?;

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
                    } else if c == 'y' {
                        FileListCommand::Yank(false)
                    } else if c == 'x' {
                        FileListCommand::Yank(true)
                    } else if c == 'p' {
                        FileListCommand::Paste
                    } else if c == 'r' {
                        FileListCommand::OmnibarMode(OmnibarMode::Rename)
                    } else if c == 't' {
                        FileListCommand::OmnibarMode(OmnibarMode::Touch)
                    } else if c == 'm' {
                        FileListCommand::OmnibarMode(OmnibarMode::Mkdir)
                    } else {
                        FileListCommand::None
                    }
                }
                KeyCode::Enter => FileListCommand::SelectEntry(
                    self.get_canonical_entry()
                        .expect("Unable to process selected entry"),
                ),
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
                if let Event::Key(ke) = event::read()? {
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
        if entry.as_path().is_dir() {
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
                .take(self.window.finder.max_entries())
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
                .filter(|i| !i.is_empty()) // Walk has directory itself as member, filter that out
                .collect::<Vec<_>>();

            if text.is_empty() {
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
                .take(self.window.finder.max_entries())
                .map(Into::into)
                .collect())
        }
    }

    fn get_canonical_entry(&self) -> Result<PathBuf> {
        let mut dir = current_dir()?;
        dir.push(PathBuf::from(self.window.file_list.curr_entry()));
        dir = dir.canonicalize()?;

        Ok(dir)
    }

    fn yank(&mut self, cut: bool) -> Result<()> {
        let path = self.get_canonical_entry()?;

        if path.is_dir() {
            return Ok(());
        }

        self.window
            .clipboard
            .push(ClipboardEntry::new(self.get_canonical_entry()?, cut));

        Ok(())
    }

    fn paste(&mut self) -> Result<()> {
        for ce in self.window.clipboard.get_files() {
            let path = ce.file().canonicalize()?;
            let mut new_path = self.curr_dir.clone();
            new_path.push(PathBuf::from(
                path.file_name().expect("Trying to copy directory"),
            ));

            fs::copy(&path, new_path)?;

            if ce.cut() {
                fs::remove_file(path)?;
            }
        }

        // Forces reload of files
        self.change_directory(self.curr_dir.clone())?;

        self.window.clipboard.clear();
        Ok(())
    }

    fn refresh_preview(&mut self) -> Result<()> {
        let path = PathBuf::from(self.window.file_list.curr_entry()).canonicalize()?;

        if !path.is_file() {
            self.window.preview.update_lines(Vec::new());

            return Ok(());
        }

        let command = Command::new("bat")
            .arg("-P")
            .arg("--wrap=never")
            .arg(format!(
                "--line-range=1:{}",
                self.window.preview.max_lines()
            ))
            .arg("--number")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?;

        // Processing it like this lets us mostly catch utf-8 issues
        self.window.preview.update_lines(
            String::from_utf8(command.wait_with_output()?.stdout)
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect(),
        );
        
        Ok(())
    }

    fn finder_mode(&mut self, zoxide: bool) -> Result<()> {
        self.window.finder_mode(true);
        self.window
            .finder
            .update_files(self.find(String::new(), zoxide)?);

        loop {
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(ke) = event::read()? {
                    if ke.kind == KeyEventKind::Press {
                        match ke.code {
                            KeyCode::Esc => break,
                            KeyCode::Char(c) => {
                                let mut text = self.window.finder.text();
                                text.push(c);
                                self.window
                                    .finder
                                    .update_files(self.find(text.clone(), zoxide)?);
                                self.window.finder.set_text(text);
                            }
                            KeyCode::Backspace => {
                                let mut text = self.window.finder.text();
                                if !text.is_empty() {
                                    text.remove(text.len() - 1);
                                    self.window
                                        .finder
                                        .update_files(self.find(text.clone(), zoxide)?);
                                    self.window.finder.set_text(text);
                                }
                            }
                            KeyCode::Enter => {
                                self.open_entry(self.window.finder.selection().into())?;
                                break;
                            }
                            KeyCode::Down | KeyCode::Tab => self.window.finder.scroll(true),
                            KeyCode::Up | KeyCode::BackTab => self.window.finder.scroll(false),
                            _ => (),
                        }
                    }
                }
            }

            self.draw()?
        }

        self.window.finder_mode(false);
        self.window.finder.update_files(Vec::new());
        self.window.finder.set_text(String::new());
        self.window.finder.reset();

        Ok(())
    }

    fn omnibar_mode(&mut self, mode: OmnibarMode) -> Result<()> {
        self.window.omnibar_mode(true);

        let mut submit = false;

        loop {
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(ke) = event::read()? {
                    if ke.kind == KeyEventKind::Press {
                        match ke.code {
                            KeyCode::Backspace => {
                                let mut text = self.window.omnibar.text().clone();
                                text.truncate(text.len().saturating_sub(1));

                                self.window.omnibar.set_text(text);
                            }
                            KeyCode::Char(c) => {
                                let mut text = self.window.omnibar.text().clone();
                                text.push(c);

                                self.window.omnibar.set_text(text);
                            }
                            KeyCode::Esc => break,
                            KeyCode::Enter => {
                                submit = true;
                                break;
                            }
                            _ => (),
                        }
                    }
                }
            }

            self.draw()?;
        }

        if submit {
            let mut newpath = self.curr_dir.clone();
            newpath.push(PathBuf::from(self.window.omnibar.text()));
            match mode {
                OmnibarMode::Rename => {
                    let entry = self.get_canonical_entry()?;
                    if entry.is_dir() {
                        return Ok(());
                    }

                    fs::copy(&entry, newpath)?;
                    fs::remove_file(entry)?;
                }
                OmnibarMode::Touch => {
                    fs::File::create(newpath)?;
                }
                OmnibarMode::Mkdir => {
                    fs::create_dir(newpath)?;
                }
            }
            self.change_directory(self.curr_dir.clone())?;
        }

        self.window.omnibar.set_text(String::new());
        self.window.omnibar_mode(false);

        Ok(())
    }

    fn execute_command(&mut self, command: FileListCommand) -> Result<()> {
        match &command {
            FileListCommand::EntryScroll(d) => self.window.file_list.scroll_entry(*d),
            FileListCommand::WindowScroll(d) => self.window.file_list.scroll_list(*d),
            FileListCommand::SelectEntry(p) => self.open_entry(p.clone())?,
            FileListCommand::Exit => self.exit = true,
            FileListCommand::HintMode => self.hint_mode()?,
            FileListCommand::FinderMode(z) => self.finder_mode(*z)?,
            FileListCommand::OmnibarMode(m) => self.omnibar_mode(*m)?,
            FileListCommand::Yank(c) => self.yank(*c)?,
            FileListCommand::Paste => self.paste()?,
            FileListCommand::None => (),
        };

        if command.should_refresh_preview() && self.refresh_preview().is_err() {
            self.window.preview.update_lines(Vec::new());
        }

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
