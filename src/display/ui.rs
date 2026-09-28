use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
    net::IpAddr,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

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
    /// A message to temporarily show in the footer, and when it was set.
    notice: Option<(Instant, String)>,
}

const NOTICE_DURATION: Duration = Duration::from_secs(5);

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
            notice: None,
        }
    }
    pub fn output_text(&mut self, write_to_stdout: &mut (dyn FnMut(&str) + Send)) {
        let opts = &self.opts;
        let show_all = !(opts.processes || opts.connections || opts.addresses);

        // header
        write_to_stdout("Refreshing:");

        // body
        for line in self.raw_lines(
            show_all || opts.processes,
            show_all || opts.connections,
            show_all || opts.addresses,
        ) {
            write_to_stdout(&line);
        }

        // footer
        write_to_stdout("");
    }

    /// Format the current state using the `--raw` output format.
    ///
    /// Yields a single `<NO TRAFFIC>` line if none of the selected sections have any data.
    fn raw_lines(&self, processes: bool, connections: bool, addresses: bool) -> Vec<String> {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        let local_time: DateTime<Local> = Local::now();
        let timestamp = local_time.timestamp();
        let mut lines = vec![];

        if processes {
            for (proc_info, process_network_data) in &state.processes {
                lines.push(format!(
                    "process: <{timestamp}> \"{}\" up/down Bps: {}/{} connections: {}",
                    proc_info.name,
                    process_network_data.total_bytes_uploaded,
                    process_network_data.total_bytes_downloaded,
                    process_network_data.connection_count
                ));
            }
        }
        if connections {
            for (connection, connection_network_data) in &state.connections {
                lines.push(format!(
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
            }
        }
        if addresses {
            for (remote_address, remote_address_network_data) in &state.remote_addresses {
                lines.push(format!(
                    "remote_address: <{timestamp}> {} up/down Bps: {}/{} connections: {}",
                    display_ip_or_host(*remote_address, ip_to_host),
                    remote_address_network_data.total_bytes_uploaded,
                    remote_address_network_data.total_bytes_downloaded,
                    remote_address_network_data.connection_count
                ));
            }
        }

        // In case no traffic is detected
        if lines.is_empty() {
            lines.push("<NO TRAFFIC>".to_string());
        }
        lines
    }

    /// Write everything currently known to a new timestamped file in `dir`,
    /// using the `--raw` output format. Returns the path of the created file.
    ///
    /// All sections are dumped, regardless of which tables are being displayed.
    pub fn dump_state(&self, dir: &Path) -> io::Result<PathBuf> {
        let file_name = format!(
            "bandwhich-dump-{}.log",
            Local::now().format("%Y%m%d-%H%M%S%.3f")
        );
        let path = dir.join(file_name);
        let mut file = File::options().write(true).create_new(true).open(&path)?;
        for line in self.raw_lines(true, true, true) {
            writeln!(file, "{line}")?;
        }
        Ok(std::path::absolute(&path).unwrap_or(path))
    }

    /// Show a short-lived message in the footer.
    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some((Instant::now(), notice));
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
                notice: self
                    .notice
                    .as_ref()
                    .filter(|(set_at, _)| set_at.elapsed() < NOTICE_DURATION)
                    .map(|(_, notice)| notice.clone()),
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
