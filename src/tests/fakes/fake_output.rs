use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use ratatui::{
    backend::{Backend, WindowSize},
    buffer::Cell,
    layout::{Position, Size},
};

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
    terminal_width: Arc<Mutex<u16>>,
    terminal_height: Arc<Mutex<u16>>,
}

impl TestBackend {
    pub fn new(
        log: Arc<Mutex<Vec<TerminalEvent>>>,
        draw_log: Arc<Mutex<Vec<String>>>,
        terminal_width: Arc<Mutex<u16>>,
        terminal_height: Arc<Mutex<u16>>,
    ) -> TestBackend {
        TestBackend {
            events: log,
            draw_events: draw_log,
            terminal_width,
            terminal_height,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct Point {
    x: u16,
    y: u16,
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

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        self.events.lock().unwrap().push(TerminalEvent::GetCursor);
        Ok(Position::new(0, 0))
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _position: P) -> io::Result<()> {
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
        }
        let terminal_height = self.terminal_height.lock().unwrap();
        let terminal_width = self.terminal_width.lock().unwrap();
        for y in 0..*terminal_height {
            for x in 0..*terminal_width {
                match coordinates.get(&Point { x, y }) {
                    Some(cell) => {
                        // this will contain no style information at all
                        // should be good enough for testing
                        string.push_str(cell.symbol());
                    }
                    None => {
                        string.push(' ');
                    }
                }
            }
            string.push('\n');
        }
        self.draw_events.lock().unwrap().push(string);
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        let terminal_height = self.terminal_height.lock().unwrap();
        let terminal_width = self.terminal_width.lock().unwrap();

        Ok(Size::new(*terminal_width, *terminal_height))
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        let width = *self.terminal_width.lock().unwrap();
        let height = *self.terminal_height.lock().unwrap();

        Ok(WindowSize {
            columns_rows: Size { width, height },
            pixels: Size::default(),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push(TerminalEvent::Flush);
        Ok(())
    }
}
