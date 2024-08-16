use std::io::stdout;

use ratatui::crossterm::{
    terminal::{disable_raw_mode, LeaveAlternateScreen},
    ExecutableCommand,
};
use thunars::{browser::Browser, tui, Result};

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|e| {
        disable_raw_mode().unwrap();
        stdout().execute(LeaveAlternateScreen).unwrap();
        println!("{}", e)
    }));

    let terminal = tui::init()?;
    let mut browser = Browser::init(terminal)?;

    browser.run()?;

    tui::restore()?;
    unimplemented!()
}
