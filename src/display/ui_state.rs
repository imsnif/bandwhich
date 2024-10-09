use std::{
    cmp,
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use log::warn;

use crate::{
    display::BandwidthUnitFamily,
    network::{Connection, LocalSocket, Utilization},
    os::ProcessInfo,
};

static RECALL_LENGTH: usize = 5;
static MAX_BANDWIDTH_ITEMS: usize = 1000;

pub trait Bandwidth {
    fn get_total_bytes_downloaded(&self) -> u128;
    fn get_total_bytes_uploaded(&self) -> u128;
    fn combine_bandwidth(&mut self, other: &Self);
    fn divide_by(&mut self, amount: u128);
}

#[derive(Clone, Default)]
pub struct NetworkData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub connection_count: u128,
}

#[derive(Clone, Default)]
pub struct ConnectionData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub process_name: String,
    pub interface_name: String,
}

impl Bandwidth for NetworkData {
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn combine_bandwidth(&mut self, other: &NetworkData) {
        self.total_bytes_downloaded += other.get_total_bytes_downloaded();
        self.total_bytes_uploaded += other.get_total_bytes_uploaded();
        self.connection_count = other.connection_count;
    }
    fn divide_by(&mut self, amount: u128) {
        self.total_bytes_downloaded /= amount;
        self.total_bytes_uploaded /= amount;
    }
}

impl Bandwidth for ConnectionData {
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn combine_bandwidth(&mut self, other: &ConnectionData) {
        self.total_bytes_downloaded += other.get_total_bytes_downloaded();
        self.total_bytes_uploaded += other.get_total_bytes_uploaded();
    }
    fn divide_by(&mut self, amount: u128) {
        self.total_bytes_downloaded /= amount;
        self.total_bytes_uploaded /= amount;
    }
}

pub struct UtilizationData {
    connections_to_procs: HashMap<LocalSocket, ProcessInfo>,
    network_utilization: Utilization,
}

#[derive(Default)]
pub struct UIState {
    /// The interface name in single-interface mode. `None` means all interfaces.
    pub interface_name: Option<String>,
    pub processes: Vec<(ProcessInfo, NetworkData)>,
    pub remote_addresses: Vec<(IpAddr, NetworkData)>,
    pub connections: Vec<(Connection, ConnectionData)>,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub cumulative_mode: bool,
    pub show_dns: bool,
    pub unit_family: BandwidthUnitFamily,
    pub utilization_data: VecDeque<UtilizationData>,
    pub processes_map: HashMap<ProcessInfo, NetworkData>,
    pub remote_addresses_map: HashMap<IpAddr, NetworkData>,
    pub connections_map: HashMap<Connection, ConnectionData>,
    /// Used for reducing logging noise.
    known_orphan_sockets: VecDeque<LocalSocket>,
}

impl UIState {
    pub fn update(
        &mut self,
        connections_to_procs: HashMap<LocalSocket, ProcessInfo>,
        network_utilization: Utilization,
    ) {
        self.utilization_data.push_back(UtilizationData {
            connections_to_procs,
            network_utilization,
        });
        if self.utilization_data.len() > RECALL_LENGTH {
            self.utilization_data.pop_front();
        }
        let mut processes: HashMap<ProcessInfo, NetworkData> = HashMap::new();
        let mut remote_addresses: HashMap<IpAddr, NetworkData> = HashMap::new();
        let mut connections: HashMap<Connection, ConnectionData> = HashMap::new();
        let mut total_bytes_downloaded: u128 = 0;
        let mut total_bytes_uploaded: u128 = 0;

        let mut seen_connections = HashSet::new();
        for state in self.utilization_data.iter().rev() {
            let connections_to_procs = &state.connections_to_procs;
            let network_utilization = &state.network_utilization;

            for (connection, connection_info) in &network_utilization.connections {
                let connection_previously_seen = !seen_connections.insert(connection);
                let connection_data = connections.entry(*connection).or_default();
                let data_for_remote_address = remote_addresses
                    .entry(connection.remote_socket.ip)
                    .or_default();
                connection_data.total_bytes_downloaded += connection_info.total_bytes_downloaded;
                connection_data.total_bytes_uploaded += connection_info.total_bytes_uploaded;
                connection_data
                    .interface_name
                    .clone_from(&connection_info.interface_name);
                data_for_remote_address.total_bytes_downloaded +=
                    connection_info.total_bytes_downloaded;
                data_for_remote_address.total_bytes_uploaded +=
                    connection_info.total_bytes_uploaded;
                if !connection_previously_seen {
                    data_for_remote_address.connection_count += 1;
                }
                total_bytes_downloaded += connection_info.total_bytes_downloaded;
                total_bytes_uploaded += connection_info.total_bytes_uploaded;

                let data_for_process = {
                    let local_socket = connection.local_socket;
                    let proc_info = get_proc_info(connections_to_procs, &local_socket);

                    // only log each orphan connection once
                    if proc_info.is_none() && !self.known_orphan_sockets.contains(&local_socket) {
                        // newer connections go in the front so that searches are faster
                        // basically recency bias
                        self.known_orphan_sockets.push_front(local_socket);
                        self.known_orphan_sockets.truncate(10_000); // arbitrary maximum backlog

                        match connections_to_procs
                            .iter()
                            .find(|(&LocalSocket { port, protocol, .. }, _)| {
                                port == local_socket.port && protocol == local_socket.protocol
                            })
                            .and_then(|(local_conn_lookalike, info)| {
                                network_utilization
                                    .connections
                                    .keys()
                                    .find(|conn| &conn.local_socket == local_conn_lookalike)
                                    .map(|conn| (conn, info))
                            }) {
                            Some((lookalike, proc_info)) => {
                                warn!(
                                    r#""{0}" owns a similar looking connection, but its local ip doesn't match."#,
                                    proc_info.name
                                );
                                warn!("Looking for: {connection:?}; found: {lookalike:?}");
                            }
                            None => {
                                warn!("Cannot determine which process owns {connection:?}");
                            }
                        };
                    }

                    let proc_info = proc_info
                        .cloned()
                        .unwrap_or_else(|| ProcessInfo::new("<UNKNOWN>", 0));
                    connection_data.process_name.clone_from(&proc_info.name);
                    processes.entry(proc_info).or_default()
                };

                data_for_process.total_bytes_downloaded += connection_info.total_bytes_downloaded;
                data_for_process.total_bytes_uploaded += connection_info.total_bytes_uploaded;
                if !connection_previously_seen {
                    data_for_process.connection_count += 1;
                }
            }
        }
        let divide_by = if self.utilization_data.is_empty() {
            1_u128
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

        if self.cumulative_mode {
            merge_bandwidth(&mut self.processes_map, processes);
            merge_bandwidth(&mut self.remote_addresses_map, remote_addresses);
            merge_bandwidth(&mut self.connections_map, connections);
            self.total_bytes_downloaded += total_bytes_downloaded / divide_by;
            self.total_bytes_uploaded += total_bytes_uploaded / divide_by;
        } else {
            self.processes_map = processes;
            self.remote_addresses_map = remote_addresses;
            self.connections_map = connections;
            self.total_bytes_downloaded = total_bytes_downloaded / divide_by;
            self.total_bytes_uploaded = total_bytes_uploaded / divide_by;
        }
        self.processes = sort_and_prune(&mut self.processes_map);
        self.remote_addresses = sort_and_prune(&mut self.remote_addresses_map);
        self.connections = sort_and_prune(&mut self.connections_map);
    }
}

fn get_proc_info<'a>(
    connections_to_procs: &'a HashMap<LocalSocket, ProcessInfo>,
    local_socket: &LocalSocket,
) -> Option<&'a ProcessInfo> {
    connections_to_procs
        // direct match
        .get(local_socket)
        // IPv4-mapped IPv6 addresses
        .or_else(|| {
            let swapped: IpAddr = match local_socket.ip {
                IpAddr::V4(v4) => v4.to_ipv6_mapped().into(),
                IpAddr::V6(v6) => v6.to_ipv4_mapped()?.into(),
            };
            connections_to_procs.get(&LocalSocket {
                ip: swapped,
                ..*local_socket
            })
        })
        // address unspecified
        .or_else(|| {
            connections_to_procs.get(&LocalSocket {
                ip: Ipv4Addr::UNSPECIFIED.into(),
                ..*local_socket
            })
        })
        .or_else(|| {
            connections_to_procs.get(&LocalSocket {
                ip: Ipv6Addr::UNSPECIFIED.into(),
                ..*local_socket
            })
        })
}

fn merge_bandwidth<K, V>(self_map: &mut HashMap<K, V>, other_map: HashMap<K, V>)
where
    K: Eq + Hash,
    V: Bandwidth,
{
    for (key, b_other) in other_map {
        self_map
            .entry(key)
            .and_modify(|b_self| b_self.combine_bandwidth(&b_other))
            .or_insert(b_other);
    }
}

fn sort_and_prune<K, V>(map: &mut HashMap<K, V>) -> Vec<(K, V)>
where
    K: Eq + Hash + Clone,
    V: Bandwidth + Clone,
{
    let mut bandwidth_list = Vec::from_iter(map.clone());
    bandwidth_list.sort_by_key(|(_, b)| {
        cmp::Reverse(b.get_total_bytes_downloaded() + b.get_total_bytes_uploaded())
    });

    if bandwidth_list.len() > MAX_BANDWIDTH_ITEMS {
        for (key, _) in &bandwidth_list[MAX_BANDWIDTH_ITEMS..] {
            map.remove(key);
        }
    }

    bandwidth_list
}
