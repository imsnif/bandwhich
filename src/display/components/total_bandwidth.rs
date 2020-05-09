use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Color, Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Paragraph, Text, Widget};

use crate::display::{DisplayBandwidth, UIState};
use tui::layout::{Constraint, Direction};

pub struct HeaderDetails<'a> {
    pub state: &'a UIState,
    pub elapsed_time: std::time::Duration,
    pub paused: bool,
}

impl<'a> HeaderDetails<'a> {
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let parts = ::tui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(rect);

        let color = if self.paused {
            Color::Yellow
        } else {
            Color::Green
        };

        if self.state.cumulative_mode {
            self.render_elapsed_time(frame, parts[2], &color)
        };
        self.render_paused(frame, parts[1], &color);
        self.render_bandwidth(frame, parts[0], &color);
    }

    fn render_bandwidth(&self, frame: &mut Frame<impl Backend>, rect: Rect, color: &Color) {
        let c_mode = self.state.cumulative_mode;
        let title_text = {
            [Text::styled(
                format!(
                    " Total Up / Down: {} / {}",
                    DisplayBandwidth {
                        bandwidth: self.state.total_bytes_uploaded as f64,
                        as_rate: !c_mode,
                    },
                    DisplayBandwidth {
                        bandwidth: self.state.total_bytes_downloaded as f64,
                        as_rate: !c_mode,
                    }
                ),
                Style::default().fg(*color).modifier(Modifier::BOLD),
            )]
        };

        Paragraph::new(title_text.iter())
            .alignment(Alignment::Left)
            .render(frame, rect);
    }

    fn render_elapsed_time(&self, frame: &mut Frame<impl Backend>, rect: Rect, color: &Color) {
        let elapsed_time_text = [Text::styled(
            format!(
                "Duration: {:02}:{:02}:{:02} ",
                self.elapsed_time.as_secs() / 3600,
                (self.elapsed_time.as_secs() % 3600) / 60,
                self.elapsed_time.as_secs() % 60
            ),
            Style::default().fg(*color).modifier(Modifier::BOLD),
        )];
        Paragraph::new(elapsed_time_text.iter())
            .alignment(Alignment::Right)
            .render(frame, rect);
    }

    fn render_paused(&self, frame: &mut Frame<impl Backend>, rect: Rect, color: &Color) {
        if self.paused {
            let paused_text = [Text::styled(
                format!("PAUSED"),
                Style::default().fg(*color).modifier(Modifier::BOLD),
            )];
            Paragraph::new(paused_text.iter())
                .alignment(Alignment::Center)
                .render(frame, rect);
        }
    }
}
