#![ allow( dead_code, unused_imports ) ]

use ::human_size::{Size};
use ::std::{thread, time};
use ::std::sync::{Arc, Mutex};
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::collections::HashMap;

use crate::traffic::{Connection};
use crate::store::NetworkUtilization;
use crate::current_connections::{CurrentConnections}; // TODO: better naming
use crate::display::{UIState, IsProcess};

use ::num_bigint::{BigUint, ToBigUint, ParseBigIntError};
use ::num_traits::{Zero, One};
use ::num_traits::cast::{ToPrimitive};

use ::netstat::*;

use ::std::io;
use ::std::io::{Write, stdout, stdin};
use ::tui::Terminal;
use ::tui::terminal::Frame;
use ::tui::backend::{Backend, TermionBackend};
use ::tui::widgets::{Widget, Block, Borders, Table, Row};
use ::tui::layout::{Layout, Constraint, Direction, Rect};
use ::tui::style::{Style, Color};
use ::termion::raw::IntoRawMode;
use ::termion::event::{Key, Event, MouseEvent};
use ::termion::input::{TermRead};

macro_rules! build_table {
    ($a:expr, $b:expr, $c:expr, $d: expr) => {
        Table::new(
                $a.into_iter(),
                $b.into_iter()
            )
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
                        display_bandwidth(&$c.total_bytes_uploaded),
                        display_bandwidth(&$c.total_bytes_downloaded)
                    )
                ].into_iter(),
                Style::default().fg(Color::White)
        )
    }
}

fn display_bandwidth (bytes_per_second: &BigUint) -> String {
    if bytes_per_second > &999999999.to_biguint().unwrap() {
        format!("{:.2}GBps", bytes_per_second.to_f64().unwrap() / 1000000000.0)
    } else if bytes_per_second > &999999.to_biguint().unwrap() {
        format!("{:.2}MBps", bytes_per_second.to_f64().unwrap() / 1000000.0)
    } else if bytes_per_second > &999.to_biguint().unwrap() { // TODO: do not do this each time
        format!("{:.2}KBps", bytes_per_second.to_f64().unwrap() / 1000.0)
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

fn render_process_table <B: Backend>(state: &UIState, frame: &mut tui::terminal::Frame<B>, rect: tui::layout::Rect) {
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

fn render_connections_table<B: Backend>(state: &UIState, frame: &mut tui::terminal::Frame<B>, rect: tui::layout::Rect)
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

fn render_remote_ip_table<B: Backend>(state: &UIState, frame: &mut tui::terminal::Frame<B>, rect: tui::layout::Rect) {
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

pub fn display_loop<B: Backend, T, Z>(mirror_utilization: &Arc<Mutex<NetworkUtilization>>, terminal: &mut Terminal<B>, create_process: &Fn(i32) -> Result<T, Box<std::error::Error>>, get_sockets_info: &Fn(AddressFamilyFlags, ProtocolFlags) -> Result<Vec<SocketInfo>, Z>) where
    T: IsProcess + std::fmt::Debug,
    Z: std::fmt::Debug
{
    let state = UIState::new(&create_process, &get_sockets_info, mirror_utilization);
    terminal.draw(|mut f| {
        let screen_horizontal_halves = split(Direction::Horizontal, f.size());
        let right_side_vertical_halves = split(Direction::Vertical, screen_horizontal_halves[1]);
        render_connections_table(&state, &mut f, screen_horizontal_halves[0]);
        render_process_table(&state, &mut f, right_side_vertical_halves[0]);
        render_remote_ip_table(&state, &mut f, right_side_vertical_halves[1]);
    }).unwrap();
    mirror_utilization.lock().unwrap().reset();
}
