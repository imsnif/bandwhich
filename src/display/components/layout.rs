use ratatui::{
    layout::{Constraint, Direction, Rect},
    Frame,
};

use crate::display::{HeaderDetails, HelpText, Table};

const FIRST_HEIGHT_BREAKPOINT: u16 = 30;
const FIRST_WIDTH_BREAKPOINT: u16 = 120;

fn top_app_and_bottom_split(rect: Rect) -> (Rect, Rect, Rect) {
    // The header and the footer each take one row, and the tables get whatever is left.
    // The terminal can be shorter than 2 rows though, in which case the header takes
    // priority, then the footer, and the tables get nothing.
    let parts = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(rect.height.min(1)),
                Constraint::Length(rect.height.saturating_sub(2)),
                Constraint::Length(rect.height.saturating_sub(1).min(1)),
            ]
            .as_ref(),
        )
        .split(rect);
    (parts[0], parts[1], parts[2])
}

pub struct Layout<'a> {
    pub header: HeaderDetails<'a>,
    pub children: Vec<Table>,
    pub footer: HelpText,
}

impl Layout<'_> {
    fn progressive_split(&self, rect: Rect, splits: Vec<Direction>) -> Vec<Rect> {
        splits
            .into_iter()
            .fold(vec![rect], |mut layout, direction| {
                let last_rect = layout.pop().unwrap();
                let halves = ratatui::layout::Layout::default()
                    .direction(direction)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(last_rect);
                layout.append(&mut halves.to_vec());
                layout
            })
    }

    fn build_two_children_layout(&self, rect: Rect) -> Vec<Rect> {
        // if there are two elements
        if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
            // if the space is not enough, we drop one element
            vec![rect]
        } else if rect.width < FIRST_WIDTH_BREAKPOINT {
            // if the horizontal space is not enough, we drop one element and we split horizontally
            self.progressive_split(rect, vec![Direction::Vertical])
        } else {
            // by default we display two elements splitting vertically
            self.progressive_split(rect, vec![Direction::Horizontal])
        }
    }

    fn build_three_children_layout(&self, rect: Rect) -> Vec<Rect> {
        // if there are three elements
        if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
            //if the space is not enough, we drop two elements
            vec![rect]
        } else if rect.height < FIRST_HEIGHT_BREAKPOINT {
            // if the vertical space is not enough, we drop one element and we split vertically
            self.progressive_split(rect, vec![Direction::Horizontal])
        } else if rect.width < FIRST_WIDTH_BREAKPOINT {
            // if the horizontal space is not enough, we drop one element and we split horizontally
            self.progressive_split(rect, vec![Direction::Vertical])
        } else {
            // default layout
            let halves = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(rect);
            let top_quarters = ratatui::layout::Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(halves[0]);

            vec![top_quarters[0], top_quarters[1], halves[1]]
        }
    }

    fn build_layout(&self, rect: Rect) -> Vec<Rect> {
        if self.children.len() == 1 {
            // if there's only one element to render, it can take the whole frame
            vec![rect]
        } else if self.children.len() == 2 {
            self.build_two_children_layout(rect)
        } else {
            self.build_three_children_layout(rect)
        }
    }

    pub fn render(&self, frame: &mut Frame, rect: Rect, table_cycle_offset: usize) {
        let (top, app, bottom) = top_app_and_bottom_split(rect);
        let layout_slots = self.build_layout(app);
        for i in 0..layout_slots.len() {
            if let Some(rect) = layout_slots.get(i) {
                if let Some(child) = self
                    .children
                    .get((i + table_cycle_offset) % self.children.len())
                {
                    child.render(frame, *rect);
                }
            }
        }
        self.header.render(frame, top);
        self.footer.render(frame, bottom);
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use ratatui::{backend::TestBackend, Terminal};

    use super::*;
    use crate::display::UIState;

    /// The app must not panic when the terminal is too short to fit both the
    /// header and the footer.
    #[test]
    fn split_does_not_panic_on_tiny_terminals() {
        for height in 0..=4 {
            let rect = Rect::new(0, 0, 80, height);
            let (top, app, bottom) = top_app_and_bottom_split(rect);

            // no row may be conjured out of thin air, nor lost
            assert_eq!(
                top.height + app.height + bottom.height,
                height,
                "rows do not add up at height {height}"
            );
            // the header takes priority, then the footer, then the tables
            assert_eq!(top.height, height.min(1), "bad header at height {height}");
            assert_eq!(
                bottom.height,
                height.saturating_sub(1).min(1),
                "bad footer at height {height}"
            );
        }
    }

    #[test]
    fn render_does_not_panic_on_tiny_terminals() {
        let state = UIState::default();
        let ip_to_host = HashMap::new();

        for height in 0..=4 {
            let layout = Layout {
                header: HeaderDetails {
                    state: &state,
                    elapsed_time: Duration::from_secs(1),
                    paused: false,
                },
                children: vec![
                    Table::create_processes_table(&state),
                    Table::create_remote_addresses_table(&state, &ip_to_host),
                    Table::create_connections_table(&state, &ip_to_host),
                ],
                footer: HelpText {
                    paused: false,
                    show_dns: false,
                },
            };

            // the backend needs a non-zero size, but the rect we render into does not
            let mut terminal = Terminal::new(TestBackend::new(80, height.max(1))).unwrap();
            terminal
                .draw(|frame| layout.render(frame, Rect::new(0, 0, 80, height), 0))
                .unwrap();
        }
    }
}
