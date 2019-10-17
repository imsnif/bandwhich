use ::std::collections::HashMap;

use ::tui::backend::Backend;
use ::tui::Terminal;

use crate::display::components::{Layout, Table, TotalBandwidth};
use crate::display::UIState;
use crate::network::{Connection, Utilization};

use ::std::net::Ipv4Addr;

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
            terminal,
            state: Default::default(),
            ip_to_host: Default::default(),
        }
    }
    pub fn draw(&mut self) {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        self.terminal
            .draw(|mut frame| {
                let size = frame.size();
                let connections = Table::create_connections_table(&state, &ip_to_host);
                let processes = Table::create_processes_table(&state);
                let remote_ips = Table::create_remote_ips_table(&state, &ip_to_host);
                let total_bandwidth = TotalBandwidth { state: &state };
                let layout = Layout {
                    header: total_bandwidth,
                    children: vec![processes, connections, remote_ips],
                };
                layout.render(&mut frame, size);
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
        self.terminal.clear().unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
