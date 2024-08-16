use std::{
    env::{current_dir, set_current_dir},
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    components::{ClipboardEntry, File, Window, BLOCK_LINES, CURR_DIR_LINES, TOTAL_USED_LINES},
    config::{Config, FileListCommand, FinderCommand, OmnibarCommand, OmnibarType},
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
    config: Config,
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
    pub fn init(terminal: Tui, config: Config) -> Result<Browser> {
        let curr_dir = std::env::current_dir()?;
        let files = fetch_files(curr_dir.as_path())?;

        Ok(Self {
            window: Window::new(files, curr_dir.to_string_lossy().to_string()),
            terminal,
            config,
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
        self.file_list_mode()
    }

    fn file_list_command(&mut self, ke: KeyEvent) -> FileListCommand {
        if ke.kind == KeyEventKind::Press {
            match self.config.get_filelist_command(ke.code) {
                Some(c) => c,
                None => FileListCommand::None,
            }
        } else {
            FileListCommand::None
        }
    }

    fn finder_command(&mut self, ke: KeyEvent) -> FinderCommand {
        if ke.kind == KeyEventKind::Press {
            match self.config.get_finder_command(ke.code) {
                Some(c) => c,
                None => FinderCommand::None,
            }
        } else {
            FinderCommand::None
        }
    }

    fn omnibar_command(&mut self, ke: KeyEvent) -> OmnibarCommand {
        if ke.kind == KeyEventKind::Press {
            match self.config.get_omnibar_command(ke.code) {
                Some(c) => c,
                None => OmnibarCommand::None,
            }
        } else {
            OmnibarCommand::None
        }
    }

    fn file_list_mode(&mut self) -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(16))? {
                if let event::Event::Key(ke) = event::read()? {
                    let command = self.file_list_command(ke);
                    self.execute_file_list_command(command)?
                }
            }

            self.draw()?;

            if self.exit {
                return Ok(());
            }
        }
    }

    // Eventually may make this configurable
    fn hint_mode(&mut self) -> Result<()> {
        self.window.hint_mode(true);
        let mut hint = String::new();

        let submit = loop {
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(ke) = event::read()? {
                    if ke.kind == KeyEventKind::Press {
                        match ke.code {
                            KeyCode::Char(c) => {
                                hint.push(c);
                                if self.window.file_list.valid_hint(&hint) {
                                    break true;
                                } else if hint.len() >= 2 {
                                    break false;
                                }
                            }
                            c => {
                                if let Some(FileListCommand::ExitHint) =
                                    self.config.get_filelist_command(c)
                                {
                                    break false;
                                }
                            }
                        }
                    }
                }
            }

            self.draw()?;
        };

        if submit {
            self.window.file_list.jump_hint(hint)
        }

        self.window.hint_mode(false);

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
                    let command = self.finder_command(ke);
                    if self.execute_finder_command(command, zoxide)? {
                        break;
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

    fn omnibar_mode(&mut self, mode: OmnibarType) -> Result<()> {
        self.window.omnibar_mode(true, mode);

        loop {
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(ke) = event::read()? {
                    let command = self.omnibar_command(ke);
                    if self.execute_omnibar_command(command, mode)? {
                        break;
                    }
                }
            }

            self.draw()?;
        }

        self.window.omnibar.set_text(String::new());
        self.window.omnibar_mode(false, mode);

        Ok(())
    }

    fn execute_file_list_command(&mut self, command: FileListCommand) -> Result<()> {
        match &command {
            FileListCommand::EntryScroll(d) => self.window.file_list.scroll_entry(*d),
            FileListCommand::SelectEntry => self.open_entry(self.get_canonical_entry()?)?,
            FileListCommand::HintMode => self.hint_mode()?,
            FileListCommand::FinderMode(z) => self.finder_mode(*z)?,
            FileListCommand::OmnibarMode(m) => self.omnibar_mode(*m)?,
            FileListCommand::Yank(c) => self.yank(*c)?,
            FileListCommand::Paste => self.paste()?,
            FileListCommand::Exit => self.exit = true,
            FileListCommand::None | FileListCommand::ExitHint => (), // hint mode handles the latter binding
        };

        if command.should_refresh_preview() && self.refresh_preview().is_err() {
            self.window.preview.update_lines(Vec::new());
        }

        Ok(())
    }

    fn execute_finder_command(&mut self, command: FinderCommand, zoxide: bool) -> Result<bool> {
        match &command {
            FinderCommand::Write(c) => {
                let mut text = self.window.finder.text();
                text.push(*c);
                self.window
                    .finder
                    .update_files(self.find(text.clone(), zoxide)?);
                self.window.finder.set_text(text);
            }
            FinderCommand::Backspace => {
                let mut text = self.window.finder.text();
                if !text.is_empty() {
                    text.remove(text.len() - 1);
                    self.window
                        .finder
                        .update_files(self.find(text.clone(), zoxide)?);
                    self.window.finder.set_text(text);
                }
            }
            FinderCommand::SelectEntry => {
                self.open_entry(self.window.finder.selection().into())?;
                return Ok(true);
            }
            FinderCommand::EntryScroll(d) => {
                self.window.finder.scroll(*d);
            }
            FinderCommand::Exit => return Ok(true),
            FinderCommand::None => (),
        }

        Ok(false)
    }

    fn execute_omnibar_command(
        &mut self,
        command: OmnibarCommand,
        mode: OmnibarType,
    ) -> Result<bool> {
        let mut submit = false;

        match command {
            OmnibarCommand::Write(c) => {
                let mut text = self.window.omnibar.text().clone();
                text.push(c);

                self.window.omnibar.set_text(text);
            }
            OmnibarCommand::Backspace => {
                let mut text = self.window.omnibar.text().clone();
                text.truncate(text.len().saturating_sub(1));

                self.window.omnibar.set_text(text);
            }
            OmnibarCommand::Submit => submit = true,
            OmnibarCommand::Exit => return Ok(true),
            OmnibarCommand::None => (),
        }

        if submit {
            let mut newpath = self.curr_dir.clone();
            newpath.push(PathBuf::from(self.window.omnibar.text()));
            match mode {
                OmnibarType::Rename => {
                    let entry = self.get_canonical_entry()?;
                    if entry.is_dir() {
                        panic!("Can't rename a directory")
                    }

                    fs::copy(&entry, newpath)?;
                    fs::remove_file(entry)?;
                }
                OmnibarType::Touch => {
                    fs::File::create(newpath)?;
                }
                OmnibarType::Mkdir => {
                    fs::create_dir(newpath)?;
                }
            }

            self.change_directory(self.curr_dir.clone())?;

            Ok(true)
        } else {
            Ok(false)
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
