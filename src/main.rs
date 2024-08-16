use std::io::stdout;

use ratatui::crossterm::{
    terminal::{disable_raw_mode, LeaveAlternateScreen},
    ExecutableCommand,
};
use thunars::{browser::Browser, config::Config, tui, Result};

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|e| {
        disable_raw_mode().unwrap();
        stdout().execute(LeaveAlternateScreen).unwrap();
        println!("{}", e)
    }));

    let terminal = tui::init().expect("Unable to create terminal object");
    let config = Config::init().expect("Unable to initialize config");
    let mut browser = Browser::init(terminal, config).expect("Unable to initialize state");

    browser.run().expect("Error running main loop");

    tui::restore().expect("Unable to restore terminal to defaults");

    Ok(())
}
