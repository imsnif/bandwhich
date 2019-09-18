use ::std::io;
use ::std::sync::{Arc, Mutex};
use ::std::collections::HashMap;
use ::tui::backend::Backend;
use ::tui::buffer::Cell;
use ::tui::layout::Rect;

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
    terminal_width: u16,
    terminal_height: u16
}

impl TestBackend {
    pub fn new(
        log: Arc<Mutex<Vec<TerminalEvent>>>,
        draw_log: Arc<Mutex<Vec<String>>>,
    ) -> TestBackend {
        TestBackend {
            events: log,
            draw_events: draw_log,
            terminal_width: 190,
            terminal_height: 50,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct Point {
    x: u16,
    y: u16
}

impl Backend for TestBackend {
    fn clear(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::Clear);
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::HideCursor);
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::ShowCursor);
        Ok(())
    }

    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        self.events.lock().unwrap().push(TerminalEvent::GetCursor);
        Ok((0, 0))
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
        Ok(())
    }

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        // use std::fmt::Write;
        self.events.lock().unwrap().push(TerminalEvent::Draw);
        let mut string = String::with_capacity(content.size_hint().0 * 3);
        let mut coordinates = HashMap::new();
        for (x, y, cell) in content {
            coordinates.insert(Point { x, y }, cell);
        };
        for y in 0..self.terminal_height {
            for x in 0..self.terminal_width {
                match coordinates.get(&Point {x, y}) {
                    Some(cell) => {
                        // this will contain no style information at all
                        // should be good enough for testing
                        string.push_str(&cell.symbol);
                    },
                    None => {
                        string.push_str(" ");
                    }
                }
            }
            string.push_str("\n");
        }
        self.draw_events.lock().unwrap().push(string);
        Ok(())
    }

    fn size(&self) -> io::Result<Rect> {
        Ok(Rect::new(0, 0, self.terminal_width, self.terminal_height))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::Flush);
        Ok(())
    }
}
