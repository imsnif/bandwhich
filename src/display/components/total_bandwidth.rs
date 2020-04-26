use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Color, Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Paragraph, Text, Widget};

use crate::display::{DisplayBandwidth, UIState};

pub struct HeaderDetails<'a> {
    pub state: &'a UIState,
    pub elapsed_time: std::time::Duration,
    pub paused: bool,
}

impl<'a> HeaderDetails<'a> {
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        self.render_bandwidth(frame, rect);
        self.render_elapsed_time(frame, rect);
    }

    fn render_bandwidth(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let c_mode = self.state.cumulative_mode;
        let title_text = {
            let paused_str = if self.paused { "[PAUSED]" } else { "" };
            let color = if self.paused {
                Color::Yellow
            } else {
                Color::Green
            };

            [Text::styled(
                format!(
                    " Total Up / Down: {} / {} {}",
                    DisplayBandwidth {
                        bandwidth: self.state.total_bytes_uploaded as f64,
                        as_rate: !c_mode,
                    },
                    DisplayBandwidth {
                        bandwidth: self.state.total_bytes_downloaded as f64,
                        as_rate: !c_mode,
                    },
                    paused_str
                ),
                Style::default().fg(color).modifier(Modifier::BOLD),
            )]
        };

        Paragraph::new(title_text.iter())
            .alignment(Alignment::Left)
            .render(frame, rect);
    }

    fn render_elapsed_time(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let elapsed_time_text = [Text::styled(
            format!(
                "Total Elapsed Time: {:02}:{:02}:{:02} ",
                self.elapsed_time.as_secs() / 3600,
                (self.elapsed_time.as_secs() % 3600) / 60,
                self.elapsed_time.as_secs() % 60
            ),
            Style::default()
                .fg(Color::LightBlue)
                .modifier(Modifier::BOLD),
        )];
        Paragraph::new(elapsed_time_text.iter())
            .alignment(Alignment::Right)
            .render(frame, rect);
    }
}
