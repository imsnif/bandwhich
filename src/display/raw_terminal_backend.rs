// this is a bit of a hack:
// the TUI backend used by this app changes stdout to raw byte mode.
// this is not desired when we do not use it (in our --raw mode),
// since it makes writing to stdout overly complex
//
// so what we do here is provide a fake backend (RawTerminalBackend)
// that implements the Backend TUI trait, but does nothing
// this way, we don't need to create the TermionBackend
// and thus skew our stdout when we don't need it

use std::io;

use ratatui::{
    backend::{Backend, WindowSize},
    buffer::Cell,
    layout::{Position, Size},
};

pub struct RawTerminalBackend {}

impl Backend for RawTerminalBackend {
    fn clear(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::new(0, 0))
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _position: P) -> io::Result<()> {
        Ok(())
    }

    fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(0, 0))
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size::default(),
            pixels: Size::default(),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
