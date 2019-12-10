use ::std::collections::HashMap;

use ::tui::backend::Backend;
use ::tui::layout::Rect;
use ::tui::style::{Color, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Block, Borders, Row, Widget};

use crate::display::{Bandwidth, DisplayBandwidth, UIState};
use crate::network::{display_connection_string, display_ip_or_host};

use ::std::net::Ipv4Addr;
use std::iter::FromIterator;

const FIRST_WIDTH_BREAKPOINT: u16 = 50;
const SECOND_WIDTH_BREAKPOINT: u16 = 71;
const THIRD_WIDTH_BREAKPOINT: u16 = 95;

const FIRST_COLUMN_WIDTHS: [u16; 4] = [20, 30, 40, 50];
const SECOND_COLUMN_WIDTHS: [u16; 1] = [20];
const THIRD_COLUMN_WIDTHS: [u16; 4] = [10, 20, 20, 20];

fn display_upload_and_download(bandwidth: &impl Bandwidth) -> String {
    format!(
        "{}/{}",
        DisplayBandwidth(bandwidth.get_total_bytes_uploaded() as f64),
        DisplayBandwidth(bandwidth.get_total_bytes_downloaded() as f64)
    )
}

fn sort_by_bandwidth<'a, T>(
    list: &'a mut Vec<(T, &impl Bandwidth)>,
) -> &'a Vec<(T, &'a impl Bandwidth)> {
    list.sort_by(|(_, a), (_, b)| {
        let a_highest = if a.get_total_bytes_downloaded() > a.get_total_bytes_uploaded() {
            a.get_total_bytes_downloaded()
        } else {
            a.get_total_bytes_uploaded()
        };
        let b_highest = if b.get_total_bytes_downloaded() > b.get_total_bytes_uploaded() {
            b.get_total_bytes_downloaded()
        } else {
            b.get_total_bytes_uploaded()
        };
        b_highest.cmp(&a_highest)
    });
    list
}

pub struct Table<'a> {
    title: &'a str,
    column_names: &'a [&'a str],
    rows: Vec<Vec<String>>,
}

impl<'a> Table<'a> {
    pub fn create_connections_table(
        state: &UIState,
        ip_to_host: &HashMap<Ipv4Addr, String>,
    ) -> Self {
        let mut connections_list = Vec::from_iter(&state.connections);
        sort_by_bandwidth(&mut connections_list);
        let connections_rows = connections_list
            .iter()
            .map(|(connection, connection_data)| {
                vec![
                    display_connection_string(&connection, &ip_to_host, &connection_data.interface),
                    connection_data.process_name.to_string(),
                    display_upload_and_download(*connection_data),
                ]
            })
            .collect();
        let connections_title = "Utilization by connection";
        let connections_column_names = &["Connection", "Process", "Rate Up/Down"];
        Table {
            title: connections_title,
            column_names: connections_column_names,
            rows: connections_rows,
        }
    }
    pub fn create_processes_table(state: &UIState) -> Self {
        let mut processes_list = Vec::from_iter(&state.processes);
        sort_by_bandwidth(&mut processes_list);
        let processes_rows = processes_list
            .iter()
            .map(|(process_name, data_for_process)| {
                vec![
                    (*process_name).to_string(),
                    data_for_process.connection_count.to_string(),
                    display_upload_and_download(*data_for_process),
                ]
            })
            .collect();
        let processes_title = "Utilization by process name";
        let processes_column_names = &["Process", "Connection count", "Rate Up/Down"];
        Table {
            title: processes_title,
            column_names: processes_column_names,
            rows: processes_rows,
        }
    }
    pub fn create_remote_addresses_table(
        state: &UIState,
        ip_to_host: &HashMap<Ipv4Addr, String>,
    ) -> Self {
        let mut remote_addresses_list = Vec::from_iter(&state.remote_addresses);
        sort_by_bandwidth(&mut remote_addresses_list);
        let remote_addresses_rows = remote_addresses_list
            .iter()
            .map(|(remote_address, data_for_remote_address)| {
                let remote_address = display_ip_or_host(**remote_address, &ip_to_host);
                vec![
                    remote_address,
                    data_for_remote_address.connection_count.to_string(),
                    display_upload_and_download(*data_for_remote_address),
                ]
            })
            .collect();
        let remote_addresses_title = "Utilization by remote address";
        let remote_addresses_column_names = &["Remote Address", "Connection Count", "Rate Up/Down"];
        Table {
            title: remote_addresses_title,
            column_names: remote_addresses_column_names,
            rows: remote_addresses_rows,
        }
    }
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        // the second column is only rendered if there is enough room for it
        // (over third breakpoint)
        let widths = if rect.width < FIRST_WIDTH_BREAKPOINT {
            vec![FIRST_COLUMN_WIDTHS[0], THIRD_COLUMN_WIDTHS[0]]
        } else if rect.width < SECOND_WIDTH_BREAKPOINT {
            vec![FIRST_COLUMN_WIDTHS[1], THIRD_COLUMN_WIDTHS[1]]
        } else if rect.width < THIRD_WIDTH_BREAKPOINT {
            vec![FIRST_COLUMN_WIDTHS[2], THIRD_COLUMN_WIDTHS[2]]
        } else {
            vec![
                FIRST_COLUMN_WIDTHS[3],
                SECOND_COLUMN_WIDTHS[0],
                THIRD_COLUMN_WIDTHS[2],
            ]
        };

        let column_names = if rect.width < THIRD_WIDTH_BREAKPOINT {
            vec![self.column_names[0], self.column_names[2]]
        } else {
            vec![
                self.column_names[0],
                self.column_names[1],
                self.column_names[2],
            ]
        };

        let rows = self.rows.iter().map(|row| {
            if rect.width < THIRD_WIDTH_BREAKPOINT {
                vec![&row[0], &row[2]]
            } else {
                vec![&row[0], &row[1], &row[2]]
            }
        });

        let table_rows =
            rows.map(|row| Row::StyledData(row.into_iter(), Style::default().fg(Color::White)));

        ::tui::widgets::Table::new(column_names.into_iter(), table_rows)
            .block(Block::default().title(self.title).borders(Borders::ALL))
            .header_style(Style::default().fg(Color::Yellow))
            .widths(&widths[..])
            .style(Style::default().fg(Color::White))
            .column_spacing(2)
            .render(frame, rect);
    }
}
