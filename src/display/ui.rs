use ::std::collections::HashMap;
use ::std::fmt;
use ::tui::backend::Backend;
use ::tui::layout::{Constraint, Direction, Layout, Rect};
use ::tui::style::{Color, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Block, Borders, Row, Table, Widget};
use ::tui::Terminal;

use crate::display::{Bandwidth, UIState};
use crate::network::{Connection, Utilization};

use ::std::net::Ipv4Addr;

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

fn display_ip_or_host(ip: &Ipv4Addr, ip_to_host: &HashMap<Ipv4Addr, String>) -> String {
    match ip_to_host.get(ip) {
        Some(host) => host.clone(),
        None => ip.to_string(),
    }
}

fn create_table<'a>(
    title: &'a str,
    column_names: &'a [&'a str],
    rows: impl Iterator<Item = Vec<String>> + 'a,
    widths: &'a [u16],
) -> impl Widget + 'a {
    let table_rows =
        rows.map(|row| Row::StyledData(row.into_iter(), Style::default().fg(Color::White)));
    Table::new(column_names.into_iter(), table_rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(widths)
        .style(Style::default().fg(Color::White))
        .column_spacing(1)
}

fn format_row_data(
    first_cell: String,
    second_cell: String,
    bandwidth: &impl Bandwidth,
) -> Vec<String> {
    vec![
        first_cell,
        second_cell,
        format!(
            "{}/{}",
            DisplayBandwidth(bandwidth.get_total_bytes_uploaded() as f64),
            DisplayBandwidth(bandwidth.get_total_bytes_downloaded() as f64)
        ),
    ]
}

fn split(direction: Direction, rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rect)
}

fn render_process_table(state: &UIState, frame: &mut Frame<impl Backend>, rect: Rect) {
    let rows = state
        .processes
        .iter()
        .map(|(process_name, data_for_process)| {
            format_row_data(
                process_name.to_string(),
                data_for_process.connection_count.to_string(),
                data_for_process,
            )
        });
    let mut table = create_table(
        "Utilization by process name",
        &["Process", "Connection Count", "Total Bytes"],
        rows,
        &[30, 30, 30],
    );
    table.render(frame, rect);
}

fn render_connections_table(
    state: &UIState,
    frame: &mut Frame<impl Backend>,
    rect: Rect,
    ip_to_host: &HashMap<Ipv4Addr, String>,
) {
    let rows = state
        .connections
        .iter()
        .map(|(connection, connection_data)| {
            let connection_string = format!(
                "{}:{} => {}:{} ({})",
                display_ip_or_host(&connection.local_socket.ip, ip_to_host),
                connection.local_socket.port,
                display_ip_or_host(&connection.remote_socket.ip, ip_to_host),
                connection.remote_socket.port,
                connection.protocol,
            );
            format_row_data(
                connection_string,
                connection_data.process_name.to_string(),
                connection_data,
            )
        });
    let mut table = create_table(
        "Utilization by connection",
        &["Connection", "Processes", "Total Bytes Up/Down"],
        rows,
        &[50, 20, 20],
    );
    table.render(frame, rect);
}

fn render_remote_ip_table(
    state: &UIState,
    frame: &mut Frame<impl Backend>,
    rect: Rect,
    ip_to_host: &HashMap<Ipv4Addr, String>,
) {
    let rows = state
        .remote_ips
        .iter()
        .map(|(remote_ip, data_for_remote_ip)| {
            let remote_ip = display_ip_or_host(remote_ip, &ip_to_host);
            format_row_data(
                remote_ip,
                data_for_remote_ip.connection_count.to_string(),
                data_for_remote_ip,
            )
        });
    let mut table = create_table(
        "Utilization by remote ip",
        &["Remote Address", "Connection Count", "Total Bytes"],
        rows,
        &[50, 20, 20],
    );
    table.render(frame, rect);
}

pub struct Ui<B>
where
    B: Backend,
{
    terminal: Terminal<B>,
    state: UIState,
    ip_to_host: HashMap<Ipv4Addr, String>,
}

impl<B> Ui<B>
where
    B: Backend,
{
    pub fn new(terminal_backend: B) -> Self {
        let mut terminal = Terminal::new(terminal_backend).unwrap();
        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();
        Ui {
            terminal: terminal,
            state: Default::default(),
            ip_to_host: Default::default(),
        }
    }
    pub fn draw(&mut self) {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        self.terminal
            .draw(|mut f| {
                let screen_horizontal_halves = split(Direction::Horizontal, f.size());
                let right_side_vertical_halves =
                    split(Direction::Vertical, screen_horizontal_halves[1]);
                render_connections_table(state, &mut f, screen_horizontal_halves[0], ip_to_host);
                render_process_table(state, &mut f, right_side_vertical_halves[0]);
                render_remote_ip_table(state, &mut f, right_side_vertical_halves[1], ip_to_host);
            })
            .unwrap();
    }
    pub fn update_state(
        &mut self,
        connections_to_procs: HashMap<Connection, String>,
        utilization: Utilization,
        ip_to_host: HashMap<Ipv4Addr, String>,
    ) {
        self.state = UIState::new(connections_to_procs, utilization);
        self.ip_to_host = ip_to_host;
    }
    pub fn end(&mut self) {
        // TODO: destroy?
        self.terminal.clear().unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
