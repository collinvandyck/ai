use crate::prelude::*;
use crossterm::execute;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;
type Error = Box<dyn std::error::Error>;

pub fn init() -> Result<Tui, Error> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

pub fn restore() -> Result<(), Error> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
