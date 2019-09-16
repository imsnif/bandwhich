// mostly copied and adjusted from https://github.com/fdehau/tui-rs/blob/master/src/backend/termion.rs

use ::std::fmt;
use ::std::io;
use ::std::sync::{Arc, Mutex};
use ::tui::backend::Backend;
use ::tui::buffer::Cell;
use ::tui::layout::Rect;
use ::tui::style;

#[derive(Hash, Debug, PartialEq)]
pub enum TerminalEvent {
    Clear,
    HideCursor,
    ShowCursor,
    GetCursor,
    Flush,
    Draw,
}

pub struct TestBackend {
    pub events: Arc<Mutex<Vec<TerminalEvent>>>,
    pub draw_events: Arc<Mutex<Vec<String>>>,
}

impl TestBackend {
    pub fn new(
        log: Arc<Mutex<Vec<TerminalEvent>>>,
        draw_log: Arc<Mutex<Vec<String>>>,
    ) -> TestBackend {
        TestBackend {
            events: log,
            draw_events: draw_log,
        }
    }
}

impl Backend for TestBackend {
    /// Clears the entire screen and move the cursor to the top left of the screen
    fn clear(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::Clear);
        Ok(())
    }

    /// Hides cursor
    fn hide_cursor(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::HideCursor);
        Ok(())
    }

    /// Shows cursor
    fn show_cursor(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::ShowCursor);
        Ok(())
    }

    /// Gets cursor position (0-based index)
    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        self.events.lock().unwrap().push(TerminalEvent::GetCursor);
        Ok((0, 0))
    }

    /// Sets cursor position (0-based index)
    fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
        Ok(())
    }

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        use std::fmt::Write;
        self.events.lock().unwrap().push(TerminalEvent::Draw);

        let mut string = String::with_capacity(content.size_hint().0 * 3);
        let mut style = style::Style::default();
        let mut last_y = 0;
        let mut last_x = 0;
        let mut inst = 0;
        for (x, y, cell) in content {
            if y != last_y || x != last_x + 1 || inst == 0 {
                write!(string, "{}", termion::cursor::Goto(x + 1, y + 1)).unwrap();
                inst += 1;
            }
            last_x = x;
            last_y = y;
            if cell.style.modifier != style.modifier {
                write!(
                    string,
                    "{}",
                    ModifierDiff {
                        from: style.modifier,
                        to: cell.style.modifier
                    }
                )
                .unwrap();
                style.modifier = cell.style.modifier;
                inst += 1;
            }
            if cell.style.fg != style.fg {
                write!(string, "{}", Fg(cell.style.fg)).unwrap();
                style.fg = cell.style.fg;
                inst += 1;
            }
            if cell.style.bg != style.bg {
                write!(string, "{}", Bg(cell.style.bg)).unwrap();
                style.bg = cell.style.bg;
                inst += 1;
            }
            string.push_str(&cell.symbol);
            inst += 1;
        }
        self.draw_events.lock().unwrap().push(string.clone());
        // uncomment this to print to screen
        //
        // write!(self.stdout, "{}", termion::clear::All)?;
        // write!(
        //     self.stdout,
        //     "{}{}{}{}",
        //     string,
        //     Fg(style::Color::Reset),
        //     Bg(style::Color::Reset),
        //     termion::style::Reset,
        // );
        Ok(())
    }

    /// Return the size of the terminal
    fn size(&self) -> io::Result<Rect> {
        Ok(Rect::new(0, 0, 190, 50))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::Flush);
        Ok(())
    }
}

struct Fg(style::Color);

struct Bg(style::Color);

struct ModifierDiff {
    from: style::Modifier,
    to: style::Modifier,
}

impl fmt::Display for Fg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use termion::color::Color;
        match self.0 {
            style::Color::Reset => termion::color::Reset.write_fg(f),
            style::Color::Black => termion::color::Black.write_fg(f),
            style::Color::Red => termion::color::Red.write_fg(f),
            style::Color::Green => termion::color::Green.write_fg(f),
            style::Color::Yellow => termion::color::Yellow.write_fg(f),
            style::Color::Blue => termion::color::Blue.write_fg(f),
            style::Color::Magenta => termion::color::Magenta.write_fg(f),
            style::Color::Cyan => termion::color::Cyan.write_fg(f),
            style::Color::Gray => termion::color::White.write_fg(f),
            style::Color::DarkGray => termion::color::LightBlack.write_fg(f),
            style::Color::LightRed => termion::color::LightRed.write_fg(f),
            style::Color::LightGreen => termion::color::LightGreen.write_fg(f),
            style::Color::LightBlue => termion::color::LightBlue.write_fg(f),
            style::Color::LightYellow => termion::color::LightYellow.write_fg(f),
            style::Color::LightMagenta => termion::color::LightMagenta.write_fg(f),
            style::Color::LightCyan => termion::color::LightCyan.write_fg(f),
            style::Color::White => termion::color::LightWhite.write_fg(f),
            style::Color::Indexed(i) => termion::color::AnsiValue(i).write_fg(f),
            style::Color::Rgb(r, g, b) => termion::color::Rgb(r, g, b).write_fg(f),
        }
    }
}
impl fmt::Display for Bg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use termion::color::Color;
        match self.0 {
            style::Color::Reset => termion::color::Reset.write_bg(f),
            style::Color::Black => termion::color::Black.write_bg(f),
            style::Color::Red => termion::color::Red.write_bg(f),
            style::Color::Green => termion::color::Green.write_bg(f),
            style::Color::Yellow => termion::color::Yellow.write_bg(f),
            style::Color::Blue => termion::color::Blue.write_bg(f),
            style::Color::Magenta => termion::color::Magenta.write_bg(f),
            style::Color::Cyan => termion::color::Cyan.write_bg(f),
            style::Color::Gray => termion::color::White.write_bg(f),
            style::Color::DarkGray => termion::color::LightBlack.write_bg(f),
            style::Color::LightRed => termion::color::LightRed.write_bg(f),
            style::Color::LightGreen => termion::color::LightGreen.write_bg(f),
            style::Color::LightBlue => termion::color::LightBlue.write_bg(f),
            style::Color::LightYellow => termion::color::LightYellow.write_bg(f),
            style::Color::LightMagenta => termion::color::LightMagenta.write_bg(f),
            style::Color::LightCyan => termion::color::LightCyan.write_bg(f),
            style::Color::White => termion::color::LightWhite.write_bg(f),
            style::Color::Indexed(i) => termion::color::AnsiValue(i).write_bg(f),
            style::Color::Rgb(r, g, b) => termion::color::Rgb(r, g, b).write_bg(f),
        }
    }
}

impl fmt::Display for ModifierDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let remove = self.from - self.to;
        if remove.contains(style::Modifier::REVERSED) {
            write!(f, "{}", termion::style::NoInvert)?;
        }
        if remove.contains(style::Modifier::BOLD) {
            // XXX: the termion NoBold flag actually enables double-underline on ECMA-48 compliant
            // terminals, and NoFaint additionally disables bold... so we use this trick to get
            // the right semantics.
            write!(f, "{}", termion::style::NoFaint)?;

            if self.to.contains(style::Modifier::DIM) {
                write!(f, "{}", termion::style::Faint)?;
            }
        }
        if remove.contains(style::Modifier::ITALIC) {
            write!(f, "{}", termion::style::NoItalic)?;
        }
        if remove.contains(style::Modifier::UNDERLINED) {
            write!(f, "{}", termion::style::NoUnderline)?;
        }
        if remove.contains(style::Modifier::DIM) {
            write!(f, "{}", termion::style::NoFaint)?;

            // XXX: the NoFaint flag additionally disables bold as well, so we need to re-enable it
            // here if we want it.
            if self.to.contains(style::Modifier::BOLD) {
                write!(f, "{}", termion::style::Bold)?;
            }
        }
        if remove.contains(style::Modifier::CROSSED_OUT) {
            write!(f, "{}", termion::style::NoCrossedOut)?;
        }
        if remove.contains(style::Modifier::SLOW_BLINK)
            || remove.contains(style::Modifier::RAPID_BLINK)
        {
            write!(f, "{}", termion::style::NoBlink)?;
        }

        let add = self.to - self.from;
        if add.contains(style::Modifier::REVERSED) {
            write!(f, "{}", termion::style::Invert)?;
        }
        if add.contains(style::Modifier::BOLD) {
            write!(f, "{}", termion::style::Bold)?;
        }
        if add.contains(style::Modifier::ITALIC) {
            write!(f, "{}", termion::style::Italic)?;
        }
        if add.contains(style::Modifier::UNDERLINED) {
            write!(f, "{}", termion::style::Underline)?;
        }
        if add.contains(style::Modifier::DIM) {
            write!(f, "{}", termion::style::Faint)?;
        }
        if add.contains(style::Modifier::CROSSED_OUT) {
            write!(f, "{}", termion::style::CrossedOut)?;
        }
        if add.contains(style::Modifier::SLOW_BLINK) || add.contains(style::Modifier::RAPID_BLINK) {
            write!(f, "{}", termion::style::Blink)?;
        }

        Ok(())
    }
}
