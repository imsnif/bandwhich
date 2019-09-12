use ::std::sync::{Arc, Mutex};

use crate::store::{CurrentConnections, NetworkUtilization};
use crate::display::UIState;

use ::tui::Terminal;
use ::tui::terminal::Frame;
use ::tui::backend::Backend;
use ::tui::widgets::{Widget, Block, Borders, Table, Row};
use ::tui::layout::{Layout, Constraint, Direction, Rect};
use ::tui::style::{Style, Color};

macro_rules! build_table {
    ($a:expr, $b:expr, $c:expr, $d: expr) => {
        Table::new($a.into_iter(), $b.into_iter())
            .block(Block::default().title($c).borders(Borders::ALL))
            .header_style(Style::default().fg(Color::Yellow))
            .widths(&$d)
            .style(Style::default().fg(Color::White))
            .column_spacing(1)
        };
}

macro_rules! build_row {
    ($a:expr, $b:expr, $c:expr) => {
        Row::StyledData(
            vec![
                $a.to_string(),
                $b,
                format!(
                    "{}/{}",
                    display_bandwidth($c.total_bytes_uploaded as f64),
                    display_bandwidth($c.total_bytes_downloaded as f64)
                )
            ].into_iter(),
            Style::default().fg(Color::White)
        )
    }
}

fn display_bandwidth (bytes_per_second: f64) -> String {
    if bytes_per_second > 999999999.0 {
        format!("{:.2}GBps", bytes_per_second / 1000000000.0)
    } else if bytes_per_second > 999999.0 {
        format!("{:.2}MBps", bytes_per_second / 1000000.0)
    } else if bytes_per_second > 999.0 {
        format!("{:.2}KBps", bytes_per_second / 1000.0)
    } else {
        format!("{}Bps", bytes_per_second)
    }
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

fn render_process_table <B: Backend>(state: &UIState, frame: &mut Frame<B>, rect: Rect) {
    let mut process_table_rows = Vec::new();
    for process_name in &state.process_names {
        let data_for_process = state.process_data.get(process_name).unwrap();
        let row = build_row!(process_name, data_for_process.connection_count.to_string(), data_for_process);
        process_table_rows.push(row);
    }
    let column_names = ["Process", "Connection Count", "Total Bytes"];
    let title = "Utilization by process name";
    let widths = [30, 30, 30];
    let mut table = build_table!(column_names, process_table_rows, title, widths);
    table.render(frame, rect);
}

fn render_connections_table<B: Backend>(state: &UIState, frame: &mut Frame<B>, rect: Rect)
{
    let mut connection_table_rows = Vec::new();
    for connection in &state.connections {
        let connection_data = state.connection_data.get(&connection).unwrap();
        let row = build_row!(connection, connection_data.processes.join(", "), connection_data);
        connection_table_rows.push(row);
    }
    let column_names = ["Connection", "Processes", "Total Bytes Up/Down"];
    let title = "Utilization by connection";
    let widths = [50, 20, 20];
    let mut table = build_table!(column_names, connection_table_rows, title, widths);
    table.render(frame, rect);
}

fn render_remote_ip_table<B: Backend>(state: &UIState, frame: &mut Frame<B>, rect: Rect) {
    let mut remote_ip_table_rows = Vec::new();
    for remote_ip in &state.remote_ips {
        let data_for_remote_ip = state.remote_ip_data.get(remote_ip).unwrap();
        let row = build_row!(remote_ip, data_for_remote_ip.connection_count.to_string(), data_for_remote_ip);
        remote_ip_table_rows.push(row);
    }
    let column_names = ["Remote Address", "Connection Count", "Total Bytes"];
    let title = "Utilization by remote ip";
    let widths = [50, 20, 20];
    let mut table = build_table!(column_names, remote_ip_table_rows, title, widths);
    table.render(frame, rect);
}

pub fn display_loop<B: Backend>(network_utilization: &Arc<Mutex<NetworkUtilization>>, terminal: &mut Terminal<B>, current_connections: CurrentConnections) {
    let state = UIState::new(current_connections, network_utilization);
    terminal.draw(|mut f| {
        let screen_horizontal_halves = split(Direction::Horizontal, f.size());
        let right_side_vertical_halves = split(Direction::Vertical, screen_horizontal_halves[1]);
        render_connections_table(&state, &mut f, screen_horizontal_halves[0]);
        render_process_table(&state, &mut f, right_side_vertical_halves[0]);
        render_remote_ip_table(&state, &mut f, right_side_vertical_halves[1]);
    }).unwrap();
    network_utilization.lock().unwrap().reset();
}
