use ::tui::Terminal;
use ::tui::terminal::Frame;
use ::tui::backend::Backend;
use ::tui::widgets::{Widget, Block, Borders, Table, Row};
use ::tui::layout::{Layout, Constraint, Direction, Rect};
use ::tui::style::{Style, Color};
use ::std::fmt;

use crate::store::{CurrentConnections, NetworkUtilization};
use crate::display::{UIState, Bandwidth};

struct DisplayBandwidth(f64);

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 > 999999999.0 {
            write!(f, "{:.2}GBps", self.0 / 1000000000.0)
        } else if self.0 > 999999.0 {
            write!(f, "{:.2}MBps", self.0 / 1000000.0)
        } else if self.0 > 999.0 {
            write!(f, "{:.2}KBps", self.0 / 1000.0)
        } else {
            write!(f, "{}Bps", self.0)
        }
    }
}

fn create_table<'a>(
    title: &'a str,
    column_names: &'a[&'a str],
    rows: impl Iterator<Item = Vec<String>> + 'a,
    widths: &'a[u16]
) -> impl Widget +'a {
    let table_rows = rows.map(
        |row| Row::StyledData(row.into_iter(),
        Style::default().fg(Color::White))
    );
    Table::new(column_names.into_iter(), table_rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(widths)
        .style(Style::default().fg(Color::White))
        .column_spacing(1)
}

fn format_row_data(first_cell: String, second_cell: String, bandwidth: &impl Bandwidth) -> Vec<String> {
    vec![
        first_cell,
        second_cell,
        format!(
            "{}/{}",
            DisplayBandwidth(bandwidth.get_total_bytes_uploaded() as f64),
            DisplayBandwidth(bandwidth.get_total_bytes_downloaded() as f64)
        )
    ]
}

fn split (direction: Direction, rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ].as_ref()
        )
        .split(rect)
}

fn render_process_table (state: &UIState, frame: &mut Frame<impl Backend>, rect: Rect) {
    let rows = state.processes.iter().map(|(process_name, data_for_process)| {
        format_row_data(process_name.to_string(), data_for_process.connection_count.to_string(), data_for_process)
    });
    let mut table = create_table(
        "Utilization by process name",
        &["Process", "Connection Count", "Total Bytes"],
        rows,
        &[30, 30, 30]
    );
    table.render(frame, rect);
}

fn render_connections_table (state: &UIState, frame: &mut Frame<impl Backend>, rect: Rect) {
    let rows = state.connections.iter().map(|(connection, connection_data)| {
        format_row_data(connection.to_string(), connection_data.processes.join(", "), connection_data)
    });
    let mut table = create_table(
        "Utilization by connection",
        &["Connection", "Processes", "Total Bytes Up/Down"],
        rows,
        &[50, 20, 20]
    );
    table.render(frame, rect);
}

fn render_remote_ip_table (state: &UIState, frame: &mut Frame<impl Backend>, rect: Rect) {
    let rows = state.remote_ips.iter().map(|(remote_ip, data_for_remote_ip)| {
        format_row_data(remote_ip.to_string(), data_for_remote_ip.connection_count.to_string(), data_for_remote_ip)
    });
    let mut table = create_table(
        "Utilization by remote ip",
        &["Remote Address", "Connection Count", "Total Bytes"],
        rows,
        &[50, 20, 20]
    );
    table.render(frame, rect);
}

pub fn display_loop(network_utilization: &NetworkUtilization, terminal: &mut Terminal<impl Backend>, current_connections: CurrentConnections) {
    let state = UIState::new(current_connections, &network_utilization);
    terminal.draw(|mut f| {
        let screen_horizontal_halves = split(Direction::Horizontal, f.size());
        let right_side_vertical_halves = split(Direction::Vertical, screen_horizontal_halves[1]);
        render_connections_table(&state, &mut f, screen_horizontal_halves[0]);
        render_process_table(&state, &mut f, right_side_vertical_halves[0]);
        render_remote_ip_table(&state, &mut f, right_side_vertical_halves[1]);
    }).unwrap();
}
