use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Color, Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Paragraph, Text, Widget};

use crate::display::{DisplayBandwidth, UIState};

pub struct TotalBandwidth<'a> {
    pub state: &'a UIState,
    pub paused: bool,
    pub interface_name: &'a Option<String>,
}

impl<'a> TotalBandwidth<'a> {
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let title_text = {
            let paused_str = if self.paused { "[PAUSED]" } else { "" };
            let color = if self.paused {
                Color::Yellow
            } else {
                Color::Green
            };

            let interface = if let Some(val) = self.interface_name.as_deref() {
                format!("{}: ", val)
            } else {
                format!("")
            };

            [Text::styled(
                format!(
                    "{}Total Rate Up / Down: {} / {} {}",
                    interface,
                    DisplayBandwidth(self.state.total_bytes_uploaded as f64),
                    DisplayBandwidth(self.state.total_bytes_downloaded as f64),
                    paused_str
                ),
                Style::default().fg(color).modifier(Modifier::BOLD),
            )]
        };
        Paragraph::new(title_text.iter())
            .alignment(Alignment::Left)
            .render(frame, rect);
    }
}
