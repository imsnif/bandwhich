use ::tui::backend::Backend;
use ::tui::layout::{Constraint, Direction, Rect};
use ::tui::terminal::Frame;

use super::Table;
use super::TotalBandwidth;

const FIRST_HEIGHT_BREAKPOINT: u16 = 30;
const FIRST_WIDTH_BREAKPOINT: u16 = 120;
const SECOND_WIDTH_BREAKPOINT: u16 = 150;

fn leave_gap_on_top_of_rect(rect: Rect) -> Rect {
    let app = ::tui::layout::Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(1), Constraint::Length(rect.height - 1)].as_ref())
        .split(rect);
    return app[1];
}

pub struct Layout<'a> {
    pub header: TotalBandwidth<'a>,
    pub children: Vec<Table<'a>>,
}

impl<'a> Layout<'a> {
    fn split_rect(&self, rect: Rect, splits: Vec<Direction>) -> Vec<Rect> {
        let mut ret = vec![rect]; // TODO: use fold
        for direction in splits {
            let last_split = ret.pop().unwrap();
            let mut halves = ::tui::layout::Layout::default()
                .direction(direction)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(last_split);
            ret.append(&mut halves);
        }
        ret
    }
    fn get_render_order(&self, rect: &Rect) -> Vec<Rect> {
        if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
            self.split_rect(*rect, vec![])
        } else if rect.height < FIRST_HEIGHT_BREAKPOINT {
            self.split_rect(*rect, vec![Direction::Horizontal])
        } else if rect.width < FIRST_WIDTH_BREAKPOINT {
            self.split_rect(*rect, vec![Direction::Vertical])
        } else if rect.width < SECOND_WIDTH_BREAKPOINT {
            self.split_rect(*rect, vec![Direction::Vertical, Direction::Horizontal])
        } else {
            self.split_rect(*rect, vec![Direction::Horizontal, Direction::Vertical])
        }
    }
    pub fn render(&self, mut frame: &mut Frame<impl Backend>, rect: Rect) {
        let app = leave_gap_on_top_of_rect(rect);
        let render_order = self.get_render_order(&app);
        for i in 0..render_order.len() {
            if let Some(rect) = render_order.get(i) {
                if let Some(child) = self.children.get(i) {
                    child.render(&mut frame, *rect);
                }
            }
        }
        self.header.render(&mut frame, rect);
    }
}
