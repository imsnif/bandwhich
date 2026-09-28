use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

pub struct HelpText {
    pub paused: bool,
    pub show_dns: bool,
    /// A temporary message, shown in place of the usual tips.
    pub notice: Option<String>,
}

const FIRST_WIDTH_BREAKPOINT: u16 = 76;
const SECOND_WIDTH_BREAKPOINT: u16 = 54;

const TEXT_WHEN_PAUSED: &str = " Press <SPACE> to resume.";
const TEXT_WHEN_NOT_PAUSED: &str = " Press <SPACE> to pause.";
const TEXT_WHEN_DNS_NOT_SHOWN: &str = " (DNS queries hidden).";
const TEXT_WHEN_DNS_SHOWN: &str = " (DNS queries shown).";
const TEXT_TAB_TIP: &str = " Use <TAB> to rearrange tables.";

impl HelpText {
    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        let pause_content = if self.paused {
            TEXT_WHEN_PAUSED
        } else {
            TEXT_WHEN_NOT_PAUSED
        };

        if let Some(notice) = &self.notice {
            let text = Span::styled(
                format!("{pause_content} {notice}"),
                Style::default().add_modifier(Modifier::BOLD),
            );
            frame.render_widget(Paragraph::new(text).alignment(Alignment::Left), rect);
            return;
        }

        let dns_content = if rect.width <= FIRST_WIDTH_BREAKPOINT {
            ""
        } else if self.show_dns {
            TEXT_WHEN_DNS_SHOWN
        } else {
            TEXT_WHEN_DNS_NOT_SHOWN
        };

        let tab_text = if rect.width <= SECOND_WIDTH_BREAKPOINT {
            ""
        } else {
            TEXT_TAB_TIP
        };

        let text = Span::styled(
            [pause_content, tab_text, dns_content].concat(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let paragraph = Paragraph::new(text).alignment(Alignment::Left);
        frame.render_widget(paragraph, rect);
    }
}
