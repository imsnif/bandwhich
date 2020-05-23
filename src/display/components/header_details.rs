use crate::display::{DisplayBandwidth, UIState};
use ::std::time::{Duration, Instant};
use ::tui::backend::Backend;
use ::tui::layout::{Alignment, Rect};
use ::tui::style::{Color, Modifier, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Paragraph, Text, Widget};

const SECONDS_IN_DAY: u64 = 86400;

pub struct HeaderDetails<'a> {
    pub state: &'a UIState,
    pub elapsed_time: std::time::Duration,
    pub paused: bool,
}

pub fn elapsed_time(last_start_time: Instant, cumulative_time: Duration, paused: bool) -> Duration {
    if paused {
        cumulative_time
    } else {
        cumulative_time + last_start_time.elapsed()
    }
}

impl<'a> HeaderDetails<'a> {
    #[allow(clippy::int_plus_one)]
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let bandwidth = self.bandwidth_string();
        let mut elapsed_time = None;
        let print_elapsed_time = if self.state.cumulative_mode {
            elapsed_time = Some(self.elapsed_time_string());
            bandwidth.len() + elapsed_time.as_ref().unwrap().len() + 1 <= rect.width as usize
        } else {
            false
        };

        let color = if self.paused {
            Color::Yellow
        } else {
            Color::Green
        };

        if print_elapsed_time {
            self.render_elapsed_time(frame, rect, elapsed_time.as_ref().unwrap(), color);
        }
        self.render_bandwidth(frame, rect, &bandwidth, color);
    }

    fn render_bandwidth(
        &self,
        frame: &mut Frame<impl Backend>,
        rect: Rect,
        bandwidth: &str,
        color: Color,
    ) {
        let bandwidth_text = {
            [Text::styled(
                bandwidth,
                Style::default().fg(color).modifier(Modifier::BOLD),
            )]
        };

        Paragraph::new(bandwidth_text.iter())
            .alignment(Alignment::Left)
            .render(frame, rect);
    }

    fn bandwidth_string(&self) -> String {
        let c_mode = self.state.cumulative_mode;
        format!(
            " Total Up / Down: {} / {}{}",
            DisplayBandwidth {
                bandwidth: self.state.total_bytes_uploaded as f64,
                as_rate: !c_mode,
            },
            DisplayBandwidth {
                bandwidth: self.state.total_bytes_downloaded as f64,
                as_rate: !c_mode,
            },
            if self.paused { " [PAUSED]" } else { "" }
        )
    }

    fn render_elapsed_time(
        &self,
        frame: &mut Frame<impl Backend>,
        rect: Rect,
        elapsed_time: &str,
        color: Color,
    ) {
        let elapsed_time_text = [Text::styled(
            elapsed_time,
            Style::default().fg(color).modifier(Modifier::BOLD),
        )];
        Paragraph::new(elapsed_time_text.iter())
            .alignment(Alignment::Right)
            .render(frame, rect);
    }

    fn days_string(&self) -> String {
        match self.elapsed_time.as_secs() / SECONDS_IN_DAY {
            0 => "".to_string(),
            1 => "1 day, ".to_string(),
            n => format!("{} days, ", n),
        }
    }

    fn elapsed_time_string(&self) -> String {
        format!(
            "{}{:02}:{:02}:{:02} ",
            self.days_string(),
            (self.elapsed_time.as_secs() % SECONDS_IN_DAY) / 3600,
            (self.elapsed_time.as_secs() % 3600) / 60,
            self.elapsed_time.as_secs() % 60
        )
    }
}
