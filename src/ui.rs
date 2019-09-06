#![ allow( dead_code, unused_imports ) ]

use ::human_size::{Size};
use ::std::{thread, time};
use ::std::sync::{Arc, Mutex};
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::collections::HashMap;

use crate::traffic::{Connection};
use crate::store::{NetworkUtilization, ConnectionData};
use crate::current_connections::{CurrentConnections}; // TODO: better naming

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


struct NetworkData {
    total_bytes_downloaded: BigUint,
    total_bytes_uploaded: BigUint,
    connection_count: BigUint
}

impl NetworkData {
    fn new () -> Self {
        NetworkData {
            total_bytes_downloaded: Zero::zero(),
            total_bytes_uploaded: Zero::zero(),
            connection_count: Zero::zero()
        }
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

pub trait IsProcess
{
    fn get_name (&self) -> String;
}

struct UIState {
    pub process_data: HashMap<String, NetworkData>,
    pub remote_ip_data: HashMap<String, NetworkData>,
    pub connection_total_bytes: HashMap<Connection, ConnectionData>,
    pub process_names: Vec<String>,
    pub connections: Vec<Connection>,
    pub remote_ips: Vec<String>
}

impl UIState {
    pub fn new <T> (current_connections: &CurrentConnections<T>, mirror_utilization: &Arc<Mutex<NetworkUtilization>>) -> Self
    where T: IsProcess + std::fmt::Debug
    // where T: IsProcess + IsProcess
    {
        let mut process_data: HashMap<String, NetworkData> = HashMap::new();
        let mut remote_ip_data: HashMap<String, NetworkData> = HashMap::new();
        let mut connection_total_bytes: HashMap<Connection, ConnectionData> = HashMap::new();
        for (connection, associated_processes) in &current_connections.connections {
            match mirror_utilization.lock().unwrap().connections.get(connection) {
                Some(connection_data) => {
                    for process in associated_processes.iter() {
                        let data_for_process = process_data 
                            // .entry(process.stat.comm.to_string())
                            .entry(process.get_name())
                            .or_insert(NetworkData::new());
                        data_for_process.total_bytes_downloaded += &connection_data.total_bytes_downloaded;
                        data_for_process.total_bytes_uploaded += &connection_data.total_bytes_uploaded;
                        data_for_process.connection_count += &One::one();
                    }
                    let connection_data_entry = connection_total_bytes
                        .entry(connection.clone())
                        .or_insert(connection_data.clone());
                    let data_for_remote_ip = remote_ip_data
                        .entry(connection.remote_ip.to_string())
                        .or_insert(NetworkData::new()); // TODO: use a ConnectionData object here and in the process as well
                    data_for_remote_ip.total_bytes_downloaded += &connection_data.total_bytes_downloaded;
                    data_for_remote_ip.total_bytes_uploaded += &connection_data.total_bytes_uploaded;
                    data_for_remote_ip.connection_count += &One::one();
                },
                None => ()
            }
        }
        let mut process_names: Vec<String> = Vec::new();
        let mut connections: Vec<Connection> = Vec::new();
        let mut remote_ips: Vec<String> = Vec::new();
        for (process_name, _) in &process_data {
            process_names.push(process_name.to_string());
        }
        for (connection, _) in &connection_total_bytes {
            connections.push(connection.clone());
        }
        for (remote_ip, _) in &remote_ip_data {
            remote_ips.push(remote_ip.to_string());
        }
        process_names.sort();
        connections.sort_by(|a, b| a.partial_cmp(b).unwrap());
        remote_ips.sort();
        UIState {
            process_data,
            remote_ip_data,
            connection_total_bytes,
            process_names,
            connections,
            remote_ips
        }
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

fn render_process_table <B: Backend>(state: &UIState, row_style: tui::style::Style, frame: &mut tui::terminal::Frame<B>, rect: tui::layout::Rect) {
    let mut process_table_rows = Vec::new();
    for process_name in &state.process_names {
        let data_for_process = state.process_data.get(process_name).unwrap();
        let up_bps = display_bandwidth(&data_for_process.total_bytes_uploaded);
        let down_bps = display_bandwidth(&data_for_process.total_bytes_downloaded);
        process_table_rows.push(Row::StyledData(
                vec![
                    process_name.to_string(),
                    data_for_process.connection_count.to_string(),
                    format!("{}/{}", up_bps, down_bps)
                ].into_iter(),
                row_style
        ));
    }
    let column_names = ["Process", "Connection Count", "Total Bytes"];
    let title = "Utilization by process name";
    let widths = [30, 30, 30];
    let mut table = build_table!(column_names, process_table_rows, title, widths);
    table.render(frame, rect);
}

fn render_connections_table<B: Backend, T>(state: &UIState, current_connections: &CurrentConnections<T>, row_style: tui::style::Style, frame: &mut tui::terminal::Frame<B>, rect: tui::layout::Rect)
where T: IsProcess + std::fmt::Debug
{
    let mut connection_table_rows = Vec::new();
    for connection in &state.connections {
        let connection_data = state.connection_total_bytes.get(&connection).unwrap();
        match current_connections.connections.get(&connection) {
            Some(associated_processes) => {
                let processes = associated_processes.iter().map(|p| p.get_name()).collect();
                let up_bps = display_bandwidth(&connection_data.total_bytes_uploaded);
                let down_bps = display_bandwidth(&connection_data.total_bytes_downloaded);
                connection_table_rows.push(Row::StyledData(
                        vec![
                            connection.to_string(),
                            processes,
                            format!("{}/{}", up_bps, down_bps)
                        ].into_iter(),
                row_style));
            },
            None => ()
        }
    }
    let column_names = ["Connection", "Processes", "Total Bytes Up/Down"];
    let title = "Utilization by connection";
    let widths = [50, 20, 20];
    let mut table = build_table!(column_names, connection_table_rows, title, widths);
    table.render(frame, rect);
}

fn render_remote_ip_table<B: Backend>(state: &UIState, row_style: tui::style::Style, frame: &mut tui::terminal::Frame<B>, rect: tui::layout::Rect) {
    let mut remote_ip_table_rows = Vec::new();
    for remote_ip in &state.remote_ips {
        let data_for_remote_ip = state.remote_ip_data.get(remote_ip).unwrap();
        let up_bps = display_bandwidth(&data_for_remote_ip.total_bytes_uploaded);
        let down_bps = display_bandwidth(&data_for_remote_ip.total_bytes_downloaded);
        remote_ip_table_rows.push(Row::StyledData(
                vec![
                    remote_ip.to_string(),
                    data_for_remote_ip.connection_count.to_string(),
                    format!("{}/{}", up_bps, down_bps)
                ].into_iter(),
                row_style
        ));
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
    let current_connections = CurrentConnections::new(create_process, get_sockets_info);
    let state = UIState::new(&current_connections, mirror_utilization);
    terminal.draw(|mut f| {
        let screen_horizontal_halves = split(Direction::Horizontal, f.size());
        let right_side_vertical_halves = split(Direction::Vertical, screen_horizontal_halves[1]);
        let row_style = Style::default().fg(Color::White);
        render_connections_table(&state, &current_connections, row_style, &mut f, screen_horizontal_halves[0]);
        render_process_table(&state, row_style, &mut f, right_side_vertical_halves[0]);
        render_remote_ip_table(&state, row_style, &mut f, right_side_vertical_halves[1]);
    }).unwrap();
    mirror_utilization.lock().unwrap().reset();
}
