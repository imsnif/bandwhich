use ::tui::backend::Backend;
use ::tui::layout::{Constraint, Direction, Rect};
use ::tui::terminal::Frame;

use super::Table;
use super::TotalBandwidth;
use super::HelpText;

const FIRST_HEIGHT_BREAKPOINT: u16 = 30;
const FIRST_WIDTH_BREAKPOINT: u16 = 120;
const SECOND_WIDTH_BREAKPOINT: u16 = 150;

fn top_app_and_bottom_split(rect: Rect) -> (Rect, Rect, Rect) {
    let parts = ::tui::layout::Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(1), Constraint::Length(rect.height - 2), Constraint::Length(1)].as_ref())
        .split(rect);
    (parts[0], parts[1], parts[2])
}

pub struct Layout<'a> {
    pub header: TotalBandwidth<'a>,
    pub children: Vec<Table<'a>>,
    pub footer: HelpText,
}

impl<'a> Layout<'a> {
    fn progressive_split(&self, rect: Rect, splits: Vec<Direction>) -> Vec<Rect> {
        splits
            .into_iter()
            .fold(vec![rect], |mut layout, direction| {
                let last_rect = layout.pop().unwrap();
                let mut halves = ::tui::layout::Layout::default()
                    .direction(direction)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(last_rect);
                layout.append(&mut halves);
                layout
            })
    }
    fn build_layout(&self, rect: Rect) -> Vec<Rect> {
        if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
            self.progressive_split(rect, vec![])
        } else if rect.height < FIRST_HEIGHT_BREAKPOINT {
            self.progressive_split(rect, vec![Direction::Horizontal])
        } else if rect.width < FIRST_WIDTH_BREAKPOINT {
            self.progressive_split(rect, vec![Direction::Vertical])
        } else if rect.width < SECOND_WIDTH_BREAKPOINT {
            self.progressive_split(rect, vec![Direction::Vertical, Direction::Horizontal])
        } else {
            self.progressive_split(rect, vec![Direction::Horizontal, Direction::Vertical])
        }
    }
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let (top, app, bottom) = top_app_and_bottom_split(rect);
        let layout_slots = self.build_layout(app);
        for i in 0..layout_slots.len() {
            if let Some(rect) = layout_slots.get(i) {
                if let Some(child) = self.children.get(i) {
                    child.render(frame, *rect);
                }
            }
        }
        self.header.render(frame, top);
        self.footer.render(frame, bottom);
    }
}
