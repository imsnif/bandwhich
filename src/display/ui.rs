use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use chrono::prelude::*;
use ratatui::{backend::Backend, Terminal};

use crate::{
    cli::{Opt, RenderOpts},
    display::{
        components::{HeaderDetails, HelpText, Layout, SlotInfo, Table},
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
    table_scroll_offsets: Vec<usize>,
    displayed_slots: Vec<SlotInfo>,
    focused_slot: usize,
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
            table_scroll_offsets: Vec::new(),
            displayed_slots: Vec::new(),
            focused_slot: 0,
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
        let children = self.get_tables_to_display();
        let table_count = children.len();
        if self.table_scroll_offsets.len() < table_count {
            self.table_scroll_offsets.resize(table_count, 0);
        }
        let focused_child = self.focused_child_index(table_cycle_offset);
        let layout = Layout {
            header: HeaderDetails {
                state: &self.state,
                elapsed_time,
                paused,
            },
            children,
            footer: HelpText {
                paused,
                show_dns: self.state.show_dns,
            },
        };
        let table_scroll_offsets = &mut self.table_scroll_offsets;
        let displayed_slots = &mut self.displayed_slots;
        self.terminal
            .draw(|frame| {
                *displayed_slots = layout.render(
                    frame,
                    frame.area(),
                    table_cycle_offset,
                    table_scroll_offsets,
                    focused_child,
                );
            })
            .unwrap();
    }

    pub fn focused_child_index(&self, table_cycle_offset: usize) -> usize {
        let count = self.get_table_count();
        if count == 0 {
            return 0;
        }
        if let Some(slot) = self.displayed_slots.get(self.focused_slot) {
            slot.child_index
        } else {
            table_cycle_offset % count
        }
    }

    pub fn next_focus(&mut self, table_cycle_offset: &Arc<AtomicUsize>) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let displayed_count = self.displayed_slots.len();
        if displayed_count > 1 && self.focused_slot + 1 < displayed_count {
            self.focused_slot += 1;
        } else {
            self.focused_slot = 0;
            let current = table_cycle_offset.load(Ordering::SeqCst);
            let next = (current + 1) % count;
            table_cycle_offset.store(next, Ordering::SeqCst);
        }
    }

    pub fn prev_focus(&mut self, table_cycle_offset: &Arc<AtomicUsize>) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let displayed_count = self.displayed_slots.len();
        if self.focused_slot > 0 {
            self.focused_slot -= 1;
        } else if displayed_count > 1 {
            self.focused_slot = displayed_count - 1;
        } else {
            let current = table_cycle_offset.load(Ordering::SeqCst);
            let prev = if current == 0 { count - 1 } else { current - 1 };
            table_cycle_offset.store(prev, Ordering::SeqCst);
        }
    }

    pub fn handle_mouse_click(&mut self, column: u16, row: u16) -> bool {
        for (i, slot) in self.displayed_slots.iter().enumerate() {
            if column >= slot.rect.x
                && column < slot.rect.x + slot.rect.width
                && row >= slot.rect.y
                && row < slot.rect.y + slot.rect.height
            {
                self.focused_slot = i;
                return true;
            }
        }
        false
    }

    pub fn handle_mouse_scroll(
        &mut self,
        column: u16,
        row: u16,
        up: bool,
        table_cycle_offset: usize,
    ) {
        let target_child_index = self
            .displayed_slots
            .iter()
            .enumerate()
            .find(|(_, slot)| {
                column >= slot.rect.x
                    && column < slot.rect.x + slot.rect.width
                    && row >= slot.rect.y
                    && row < slot.rect.y + slot.rect.height
            })
            .map(|(i, slot)| {
                self.focused_slot = i;
                slot.child_index
            })
            .unwrap_or_else(|| self.focused_child_index(table_cycle_offset));

        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        let step = 2;
        if up {
            self.table_scroll_offsets[target_child_index] =
                self.table_scroll_offsets[target_child_index].saturating_sub(step);
        } else {
            self.table_scroll_offsets[target_child_index] =
                self.table_scroll_offsets[target_child_index].saturating_add(step);
        }
    }

    pub fn scroll_down(&mut self, table_cycle_offset: usize) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let active_index = self.focused_child_index(table_cycle_offset);
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        self.table_scroll_offsets[active_index] =
            self.table_scroll_offsets[active_index].saturating_add(1);
    }

    pub fn scroll_up(&mut self, table_cycle_offset: usize) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let active_index = self.focused_child_index(table_cycle_offset);
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        self.table_scroll_offsets[active_index] =
            self.table_scroll_offsets[active_index].saturating_sub(1);
    }

    pub fn page_down(&mut self, table_cycle_offset: usize) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let active_index = self.focused_child_index(table_cycle_offset);
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        let step = self
            .terminal
            .size()
            .map(|s| (s.height as usize).saturating_sub(5).max(1))
            .unwrap_or(10);
        self.table_scroll_offsets[active_index] =
            self.table_scroll_offsets[active_index].saturating_add(step);
    }

    pub fn page_up(&mut self, table_cycle_offset: usize) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let active_index = self.focused_child_index(table_cycle_offset);
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        let step = self
            .terminal
            .size()
            .map(|s| (s.height as usize).saturating_sub(5).max(1))
            .unwrap_or(10);
        self.table_scroll_offsets[active_index] =
            self.table_scroll_offsets[active_index].saturating_sub(step);
    }

    pub fn scroll_to_top(&mut self, table_cycle_offset: usize) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let active_index = self.focused_child_index(table_cycle_offset);
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        self.table_scroll_offsets[active_index] = 0;
    }

    pub fn scroll_to_bottom(&mut self, table_cycle_offset: usize) {
        let count = self.get_table_count();
        if count == 0 {
            return;
        }
        let active_index = self.focused_child_index(table_cycle_offset);
        if self.table_scroll_offsets.len() < count {
            self.table_scroll_offsets.resize(count, 0);
        }
        self.table_scroll_offsets[active_index] = usize::MAX;
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
