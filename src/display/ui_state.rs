use crate::os::ProcessPid;
use ::std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use ::std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::network::{Connection, LocalSocket, Utilization};

static RECALL_LENGTH: usize = 5;

pub trait Bandwidth {
    fn get_total_bytes_downloaded(&self) -> u128;
    fn get_total_bytes_uploaded(&self) -> u128;
}

#[derive(Default)]
pub struct NetworkData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub connection_count: u128,
}

#[derive(Default)]
pub struct ConnectionData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub process_name: String,
    pub interface_name: String,
}

impl NetworkData {
    pub fn divide_by(&mut self, amount: u128) {
        self.total_bytes_downloaded /= amount;
        self.total_bytes_uploaded /= amount;
    }
}

impl ConnectionData {
    pub fn divide_by(&mut self, amount: u128) {
        self.total_bytes_downloaded /= amount;
        self.total_bytes_uploaded /= amount;
    }
}

impl Bandwidth for ConnectionData {
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
}

impl Bandwidth for NetworkData {
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
}

pub struct UtilizationData {
    connections_to_procs: HashMap<LocalSocket, ProcessPid>,
    network_utilization: Utilization,
}

#[derive(Default)]
pub struct UIState {
    pub processes: BTreeMap<ProcessPid, NetworkData>,
    pub remote_addresses: BTreeMap<IpAddr, NetworkData>,
    pub connections: BTreeMap<Connection, ConnectionData>,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    utilization_data: VecDeque<UtilizationData>,
}

impl UIState {
    fn get_proc_info<'a>(
        connections_to_procs: &'a HashMap<LocalSocket, ProcessPid>,
        local_socket: &LocalSocket,
    ) -> Option<&'a ProcessPid> {
        if let Some(proc_pid) = connections_to_procs.get(local_socket) {
            Some(&proc_pid)
        } else if let Some(proc_pid) = connections_to_procs.get(&LocalSocket {
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: local_socket.port,
            protocol: local_socket.protocol,
        }) {
            Some(&proc_pid)
        } else {
            if let Some(proc_pid) = connections_to_procs.get(&LocalSocket {
                ip: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                port: local_socket.port,
                protocol: local_socket.protocol,
            }) {
                Some(&proc_pid)
            } else {
                None
            }
        }
    }
    pub fn update(
        &mut self,
        connections_to_procs: HashMap<LocalSocket, ProcessPid>,
        network_utilization: Utilization,
    ) {
        self.utilization_data.push_back(UtilizationData {
            connections_to_procs,
            network_utilization,
        });
        if self.utilization_data.len() > RECALL_LENGTH {
            self.utilization_data.pop_front();
        }
        let mut processes: BTreeMap<ProcessPid, NetworkData> = BTreeMap::new();
        let mut remote_addresses: BTreeMap<IpAddr, NetworkData> = BTreeMap::new();
        let mut connections: BTreeMap<Connection, ConnectionData> = BTreeMap::new();
        let mut total_bytes_downloaded: u128 = 0;
        let mut total_bytes_uploaded: u128 = 0;

        let mut seen_connections = HashSet::new();
        for state in self.utilization_data.iter().rev() {
            let connections_to_procs = &state.connections_to_procs;
            let network_utilization = &state.network_utilization;
            for (connection, connection_info) in &network_utilization.connections {
                let connection_previously_seen = !seen_connections.insert(connection);
                let connection_data = connections.entry(connection.clone()).or_default();
                let data_for_remote_address = remote_addresses
                    .entry(connection.remote_socket.ip)
                    .or_default();
                connection_data.total_bytes_downloaded += connection_info.total_bytes_downloaded;
                connection_data.total_bytes_uploaded += connection_info.total_bytes_uploaded;
                connection_data.interface_name = connection_info.interface_name.clone();
                data_for_remote_address.total_bytes_downloaded +=
                    connection_info.total_bytes_downloaded;
                data_for_remote_address.total_bytes_uploaded +=
                    connection_info.total_bytes_uploaded;
                if !connection_previously_seen {
                    data_for_remote_address.connection_count += 1;
                }
                total_bytes_downloaded += connection_info.total_bytes_downloaded;
                total_bytes_uploaded += connection_info.total_bytes_uploaded;

                let data_for_process = if let Some(proc_info) =
                    UIState::get_proc_info(&connections_to_procs, &connection.local_socket)
                {
                    connection_data.process_name = proc_info.procname.clone();
                    processes
                        .entry(ProcessPid {
                            procname: proc_info.procname.clone(),
                            pid: proc_info.pid,
                        })
                        .or_default()
                } else {
                    connection_data.process_name = String::from("<UNKNOWN>");
                    processes
                        .entry(ProcessPid {
                            procname: connection_data.process_name.clone(),
                            pid: 0,
                        })
                        .or_default()
                };

                data_for_process.total_bytes_downloaded += connection_info.total_bytes_downloaded;
                data_for_process.total_bytes_uploaded += connection_info.total_bytes_uploaded;
                if !connection_previously_seen {
                    data_for_process.connection_count += 1;
                }
            }
        }
        let divide_by = if self.utilization_data.is_empty() {
            1 as u128
        } else {
            self.utilization_data.len() as u128
        };
        for (_, network_data) in processes.iter_mut() {
            network_data.divide_by(divide_by)
        }
        for (_, network_data) in remote_addresses.iter_mut() {
            network_data.divide_by(divide_by)
        }
        for (_, connection_data) in connections.iter_mut() {
            connection_data.divide_by(divide_by)
        }
        self.processes = processes;
        self.remote_addresses = remote_addresses;
        self.connections = connections;
        self.total_bytes_downloaded = total_bytes_downloaded / divide_by;
        self.total_bytes_uploaded = total_bytes_uploaded / divide_by;
    }
}
