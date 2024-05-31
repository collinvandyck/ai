#![allow(unused)]
use ai::{prelude::*, tui};
use std::{io, panic::catch_unwind};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Draw(io::Error),
}

fn main() -> Result<(), Error> {
    let mut term = tui::init()?;
    catch_unwind(|| {
        let _ = tui::restore();
    });
    let res = App::default().run(&mut term);
    tui::restore()?;
    res
}

#[derive(Debug, Default)]
struct App {
    counter: u8,
    exit: bool,
}

impl App {
    fn run(&mut self, term: &mut tui::Tui) -> Result<(), Error> {
        while !self.exit {
            term.draw(|frame| self.render_frame(frame))
                .map_err(Error::Draw)?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        todo!()
    }
    fn handle_events(&mut self) -> Result<(), Error> {
        todo!()
    }
}
