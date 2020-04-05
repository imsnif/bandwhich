use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Paragraph, Text, Widget};

pub struct HelpText {
    pub paused: bool,
    pub show_dns: bool,
}

const TEXT_WHEN_PAUSED: &str = " Press <SPACE> to resume";
const TEXT_WHEN_NOT_PAUSED: &str = " Press <SPACE> to pause";
const TEXT_WHEN_DNS_NOT_SHOWN: &str = " (DNS queries hidden).";
const TEXT_WHEN_DNS_SHOWN: &str = " (DNS queries shown).";

impl HelpText {
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let text = {
            let pause_content = if self.paused {
                TEXT_WHEN_PAUSED
            } else {
                TEXT_WHEN_NOT_PAUSED
            };

            let dns_content = if self.show_dns {
                TEXT_WHEN_DNS_SHOWN
            } else {
                TEXT_WHEN_DNS_NOT_SHOWN
            };

            [Text::styled(
                format!("{}{}", pause_content, dns_content),
                Style::default().modifier(Modifier::BOLD),
            )]
        };
        Paragraph::new(text.iter())
            .alignment(Alignment::Left)
            .render(frame, rect);
    }
}
