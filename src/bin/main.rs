#![allow(unused)]
use ai::{prelude::*, tui};
use ratatui::{layout::Alignment, symbols::border, text::Text, widgets::Borders};
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
        frame.render_widget(self, frame.size())
    }
    fn handle_events(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Counter App Tutorial".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(ratatui::widgets::block::Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let counter_text = Text::from(vec![Line::from(vec![
            "Value".into(),
            self.counter.to_string().yellow(),
        ])]);
        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
