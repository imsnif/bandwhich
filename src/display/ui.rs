use ::std::collections::HashMap;

use ::tui::backend::Backend;
use ::tui::Terminal;

use crate::display::components::{HelpText, Layout, Table, TotalBandwidth};
use crate::display::UIState;
use crate::network::{display_connection_string, display_ip_or_host, LocalSocket, Utilization};

use ::std::net::IpAddr;

use crate::RenderOpts;
use chrono::prelude::*;

pub struct Ui<B>
where
    B: Backend,
{
    terminal: Terminal<B>,
    state: UIState,
    ip_to_host: HashMap<IpAddr, String>,
    opts: RenderOpts,
}

impl<B> Ui<B>
where
    B: Backend,
{
    pub fn new(terminal_backend: B, opts: RenderOpts) -> Self {
        let mut terminal = Terminal::new(terminal_backend).unwrap();
        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();
        let mut state: UIState = Default::default();
        state.cumulative_mode = opts.total_utilization;
        Ui {
            terminal,
            state,
            ip_to_host: Default::default(),
            opts,
        }
    }
    pub fn output_text(&mut self, write_to_stdout: &mut (dyn FnMut(String) + Send)) {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        let local_time: DateTime<Local> = Local::now();
        let timestamp = local_time.timestamp();
        for (process, process_network_data) in &state.processes {
            write_to_stdout(format!(
                "process: <{}> \"{}\" up/down Bps: {}/{} connections: {}",
                timestamp,
                process,
                process_network_data.total_bytes_uploaded,
                process_network_data.total_bytes_downloaded,
                process_network_data.connection_count
            ));
        }
        for (connection, connection_network_data) in &state.connections {
            write_to_stdout(format!(
                "connection: <{}> {} up/down Bps: {}/{} process: \"{}\"",
                timestamp,
                display_connection_string(
                    connection,
                    ip_to_host,
                    &connection_network_data.interface_name
                ),
                connection_network_data.total_bytes_uploaded,
                connection_network_data.total_bytes_downloaded,
                connection_network_data.process_name
            ));
        }
        for (remote_address, remote_address_network_data) in &state.remote_addresses {
            write_to_stdout(format!(
                "remote_address: <{}> {} up/down Bps: {}/{} connections: {}",
                timestamp,
                display_ip_or_host(*remote_address, ip_to_host),
                remote_address_network_data.total_bytes_uploaded,
                remote_address_network_data.total_bytes_downloaded,
                remote_address_network_data.connection_count
            ));
        }
    }
    pub fn draw(&mut self, paused: bool, show_dns: bool) {
        let state = &self.state;
        let children = self.get_tables_to_display();
        self.terminal
            .draw(|mut frame| {
                let size = frame.size();
                let total_bandwidth = TotalBandwidth {
                    state: &state,
                    paused,
                };
                let help_text = HelpText { paused, show_dns };
                let layout = Layout {
                    header: total_bandwidth,
                    children,
                    footer: help_text,
                };
                layout.render(&mut frame, size);
            })
            .unwrap();
    }

    fn get_tables_to_display(&self) -> Vec<Table<'static>> {
        let opts = &self.opts;
        let mut children: Vec<Table> = Vec::new();
        if opts.processes {
            children.push(Table::create_processes_table(&self.state));
        }
        if opts.addresses {
            children.push(Table::create_remote_addresses_table(
                &self.state,
                &self.ip_to_host,
            ));
        }
        if opts.connections {
            children.push(Table::create_connections_table(
                &self.state,
                &self.ip_to_host,
            ));
        }
        if !(opts.processes || opts.addresses || opts.connections) {
            children = vec![
                Table::create_processes_table(&self.state),
                Table::create_remote_addresses_table(&self.state, &self.ip_to_host),
                Table::create_connections_table(&self.state, &self.ip_to_host),
            ];
        }
        children
    }
    pub fn update_state(
        &mut self,
        connections_to_procs: HashMap<LocalSocket, String>,
        utilization: Utilization,
        ip_to_host: HashMap<IpAddr, String>,
    ) {
        self.state.update(connections_to_procs, utilization);
        self.ip_to_host.extend(ip_to_host);
    }
    pub fn end(&mut self) {
        self.terminal.clear().unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
