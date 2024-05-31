#![allow(unused)]
use ai::{errors, prelude::*, tui};
use crossterm::event::KeyEvent;
use ratatui::{layout::Alignment, symbols::border, text::Text, widgets::Borders};
use std::{io, panic::catch_unwind};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Draw(io::Error),

    #[error(transparent)]
    ReadEvent(io::Error),

    #[error(transparent)]
    Eyre(#[from] color_eyre::Report),
}

fn main() -> Result<(), Error> {
    errors::install_hooks()?;
    let mut term = tui::init()?;
    App::default().run(&mut term)?;
    tui::restore()?;
    Ok(())
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
        match event::read().map_err(Error::ReadEvent)? {
            Event::Key(ke) if ke.kind == KeyEventKind::Press => self.handle_key_event(ke),
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, ke: KeyEvent) {
        match ke.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('j') => self.decrement_counter(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('k') => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        let (v, _) = self.counter.overflowing_add(1);
        self.counter = v;
    }

    fn decrement_counter(&mut self) {
        if true {
            self.counter -= 1;
        }
        let (v, _) = self.counter.overflowing_sub(1);
        self.counter = v;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
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
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);
        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);
        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.exit, true);
        Ok(())
    }
}
