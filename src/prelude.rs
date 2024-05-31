pub use crossterm::event::Event;
pub use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
pub use ratatui::{
    prelude::{CrosstermBackend, Rect, Stylize, Terminal},
    widgets::Paragraph,
    Frame,
};
pub use std::io::stdout;
pub use std::{error::Error, io::Stdout, time::Duration};
