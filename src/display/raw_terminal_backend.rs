// this is a bit of a hack:
// the TUI backend used by this app changes stdout to raw byte mode.
// this is not desired when we do not use it (in our --raw mode),
// since it makes writing to stdout overly complex
//
// so what we do here is provide a fake backend (RawTerminalBackend)
// that implements the Backend TUI trait, but does nothing
// this way, we don't need to create the TermionBackend
// and thus skew our stdout when we don't need it
use ::ratatui::backend::Backend;
use ::ratatui::buffer::Cell;
use ::ratatui::layout::Rect;
use ::std::io;

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

    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        Ok((0, 0))
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
        Ok(())
    }

    fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Ok(())
    }

    fn size(&self) -> io::Result<Rect> {
        Ok(Rect::new(0, 0, 0, 0))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
