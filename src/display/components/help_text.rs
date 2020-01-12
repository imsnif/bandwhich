use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Paragraph, Text, Widget};

pub struct HelpText {
    pub paused: bool,
}

const TEXT_WHEN_PAUSED: &str = " Press <SPACE> to resume.";
const TEXT_WHEN_NOT_PAUSED: &str = " Press <SPACE> to pause.";

impl HelpText {
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let text = {
            let content = if self.paused {
                TEXT_WHEN_PAUSED
            } else {
                TEXT_WHEN_NOT_PAUSED
            };

            [Text::styled(
                content,
                Style::default().modifier(Modifier::BOLD),
            )]
        };
        Paragraph::new(text.iter())
            .alignment(Alignment::Left)
            .render(frame, rect);
    }
}
