use std::{collections::HashMap, net::IpAddr, time::Duration};

use chrono::prelude::*;
use ratatui::{backend::Backend, Terminal};

use crate::{
    cli::{Opt, RenderOpts},
    display::{
        components::{HeaderDetails, HelpText, Layout, Table},
        UIState,
    },
    network::{display_connection_string, display_ip_or_host, LocalSocket, Utilization},
    os::ProcessInfo,
};

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
    pub fn new(terminal_backend: B, opts: &Opt) -> Self {
        let mut terminal = Terminal::new(terminal_backend).unwrap();
        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();
        let state = {
            let mut state = UIState::default();
            state.interface_name.clone_from(&opts.interface);
            state.unit_family = opts.render_opts.unit_family.into();
            state.cumulative_mode = opts.render_opts.total_utilization;
            state.show_dns = opts.show_dns;
            state
        };
        Ui {
            terminal,
            state,
            ip_to_host: Default::default(),
            opts: opts.render_opts,
        }
    }
    pub fn output_text(&mut self, write_to_stdout: &mut (dyn FnMut(&str) + Send)) {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        let local_time: DateTime<Local> = Local::now();
        let timestamp = local_time.timestamp();
        let mut no_traffic = true;

        let output_process_data = |write_to_stdout: &mut (dyn FnMut(&str) + Send),
                                   no_traffic: &mut bool| {
            for (proc_info, process_network_data) in &state.processes {
                write_to_stdout(&format!(
                    "process: <{timestamp}> \"{}\" up/down Bps: {}/{} connections: {}",
                    proc_info.name,
                    process_network_data.total_bytes_uploaded,
                    process_network_data.total_bytes_downloaded,
                    process_network_data.connection_count
                ));
                *no_traffic = false;
            }
        };

        let output_connections_data =
            |write_to_stdout: &mut (dyn FnMut(&str) + Send), no_traffic: &mut bool| {
                for (connection, connection_network_data) in &state.connections {
                    write_to_stdout(&format!(
                        "connection: <{timestamp}> {} up/down Bps: {}/{} process: \"{}\"",
                        display_connection_string(
                            connection,
                            ip_to_host,
                            &connection_network_data.interface_name,
                        ),
                        connection_network_data.total_bytes_uploaded,
                        connection_network_data.total_bytes_downloaded,
                        connection_network_data.process_name
                    ));
                    *no_traffic = false;
                }
            };

        let output_adressess_data = |write_to_stdout: &mut (dyn FnMut(&str) + Send),
                                     no_traffic: &mut bool| {
            for (remote_address, remote_address_network_data) in &state.remote_addresses {
                write_to_stdout(&format!(
                    "remote_address: <{timestamp}> {} up/down Bps: {}/{} connections: {}",
                    display_ip_or_host(*remote_address, ip_to_host),
                    remote_address_network_data.total_bytes_uploaded,
                    remote_address_network_data.total_bytes_downloaded,
                    remote_address_network_data.connection_count
                ));
                *no_traffic = false;
            }
        };

        // header
        write_to_stdout("Refreshing:");

        // body1
        if self.opts.processes {
            output_process_data(write_to_stdout, &mut no_traffic);
        }
        if self.opts.connections {
            output_connections_data(write_to_stdout, &mut no_traffic);
        }
        if self.opts.addresses {
            output_adressess_data(write_to_stdout, &mut no_traffic);
        }
        if !(self.opts.processes || self.opts.connections || self.opts.addresses) {
            output_process_data(write_to_stdout, &mut no_traffic);
            output_connections_data(write_to_stdout, &mut no_traffic);
            output_adressess_data(write_to_stdout, &mut no_traffic);
        }

        // body2: In case no traffic is detected
        if no_traffic {
            write_to_stdout("<NO TRAFFIC>");
        }

        // footer
        write_to_stdout("");
    }

    pub fn draw(&mut self, paused: bool, elapsed_time: Duration, table_cycle_offset: usize) {
        let layout = Layout {
            header: HeaderDetails {
                state: &self.state,
                elapsed_time,
                paused,
            },
            children: self.get_tables_to_display(),
            footer: HelpText {
                paused,
                show_dns: self.state.show_dns,
            },
        };
        self.terminal
            .draw(|frame| layout.render(frame, frame.area(), table_cycle_offset))
            .unwrap();
    }

    fn get_tables_to_display(&self) -> Vec<Table> {
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

    pub fn get_table_count(&self) -> usize {
        self.get_tables_to_display().len()
    }

    pub fn update_state(
        &mut self,
        connections_to_procs: HashMap<LocalSocket, ProcessInfo>,
        utilization: Utilization,
        ip_to_host: HashMap<IpAddr, String>,
    ) {
        self.state.update(connections_to_procs, utilization);
        self.ip_to_host.extend(ip_to_host);
    }
    pub fn end(&mut self) {
        self.terminal.show_cursor().unwrap();
    }
}
