use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Color, Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Block, Borders, Paragraph, Text, Widget};

use crate::display::{DisplayBandwidth, UIState};

pub struct TotalBandwidth<'a> {
    pub state: &'a UIState,
}

impl<'a> TotalBandwidth<'a> {
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let title_text = [Text::styled(
            format!(
                " Total Rate Up/Down: {}/{}",
                DisplayBandwidth(self.state.total_bytes_uploaded as f64),
                DisplayBandwidth(self.state.total_bytes_downloaded as f64)
            ),
            Style::default().fg(Color::Green).modifier(Modifier::BOLD),
        )];
        Paragraph::new(title_text.iter())
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left)
            .render(frame, rect);
    }
}
