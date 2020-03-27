use ::std::cmp;
use ::std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use ::std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::iter::FromIterator;

use crate::network::{Connection, LocalSocket, Utilization};

static RECALL_LENGTH: usize = 5;
static MAX_BANDWIDTH_ITEMS: usize = 1000; // Would this be better suited as a `const`?

pub trait Bandwidth {
    fn get_total_bytes_downloaded(&self) -> u128;
    fn get_total_bytes_uploaded(&self) -> u128;
    fn combine_bandwidth(&mut self, other: &impl Bandwidth);
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

// These impls seem super repetitive, should we add either a macro or make Bandwidth
// a nested struct in NetworkData and ConnectionData?
impl Bandwidth for ConnectionData {
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn combine_bandwidth(&mut self, other: &impl Bandwidth) {
        self.total_bytes_downloaded += other.get_total_bytes_downloaded();
        self.total_bytes_uploaded += other.get_total_bytes_uploaded();
    }
}

impl Bandwidth for NetworkData {
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn combine_bandwidth(&mut self, other: &impl Bandwidth) {
        self.total_bytes_downloaded += other.get_total_bytes_downloaded();
        self.total_bytes_uploaded += other.get_total_bytes_uploaded();
    }
}

pub struct UtilizationData {
    connections_to_procs: HashMap<LocalSocket, String>,
    network_utilization: Utilization,
}

#[derive(Default)]
pub struct UIState {
    // We aren't really taking advantage of the self-sorting of the BTreeMap here,
    // but it would be rather difficult to, as it sorts by key, not value. Perhaps we
    // should just use a HashMap?
    // We could also use a HashMap internally for insertions and merging, but cache a
    // sorted vector so that doesn't need sorting during the display step. This would
    // also fix Bandwhich eating CPU cycles when paused, but is a bit of a non-issue
    // with `MAX_BANDWIDTH_ITEMS` in place.
    pub processes: BTreeMap<String, NetworkData>,
    pub remote_addresses: BTreeMap<IpAddr, NetworkData>,
    pub connections: BTreeMap<Connection, ConnectionData>,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub cumulative_mode: bool,
    pub utilization_data: VecDeque<UtilizationData>, // This needs to be public for the struct-update syntax
}

impl UIState {
    fn get_proc_name<'a>(
        connections_to_procs: &'a HashMap<LocalSocket, String>,
        local_socket: &LocalSocket,
    ) -> Option<&'a String> {
        if let Some(process_name) = connections_to_procs.get(local_socket) {
            Some(process_name)
        } else if let Some(process_name) = connections_to_procs.get(&LocalSocket {
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: local_socket.port,
            protocol: local_socket.protocol,
        }) {
            Some(process_name)
        } else {
            connections_to_procs.get(&LocalSocket {
                ip: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                port: local_socket.port,
                protocol: local_socket.protocol,
            })
        }
    }
    pub fn update(
        &mut self,
        connections_to_procs: HashMap<LocalSocket, String>,
        network_utilization: Utilization,
    ) {
        self.utilization_data.push_back(UtilizationData {
            connections_to_procs,
            network_utilization,
        });
        if self.utilization_data.len() > RECALL_LENGTH {
            self.utilization_data.pop_front();
        }
        let mut processes: BTreeMap<String, NetworkData> = BTreeMap::new();
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

                let data_for_process = if let Some(process_name) =
                    UIState::get_proc_name(&connections_to_procs, &connection.local_socket)
                {
                    connection_data.process_name = process_name.clone();
                    processes.entry(process_name.clone()).or_default()
                } else {
                    connection_data.process_name = String::from("<UNKNOWN>");
                    processes
                        .entry(connection_data.process_name.clone())
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

        if self.cumulative_mode {
            merge_bandwidth(&mut self.processes, processes);
            merge_bandwidth(&mut self.remote_addresses, remote_addresses);
            merge_bandwidth(&mut self.connections, connections);
            self.total_bytes_downloaded += total_bytes_downloaded / divide_by;
            self.total_bytes_uploaded += total_bytes_uploaded / divide_by;
        } else {
            self.processes = processes;
            self.remote_addresses = remote_addresses;
            self.connections = connections;
            self.total_bytes_downloaded = total_bytes_downloaded / divide_by;
            self.total_bytes_uploaded = total_bytes_uploaded / divide_by;
        }
        prune_map(&mut self.processes);
        prune_map(&mut self.remote_addresses);
        prune_map(&mut self.connections);
    }
}

fn merge_bandwidth<K, V>(self_map: &mut BTreeMap<K, V>, other_map: BTreeMap<K, V>)
where
    K: Eq + Ord,
    V: Bandwidth,
{
    for (key, b_other) in other_map {
        self_map
            .entry(key)
            .and_modify(|b_self| b_self.combine_bandwidth(&b_other))
            .or_insert(b_other);
    }
}

fn prune_map(map: &mut BTreeMap<impl Eq + Ord + Clone, impl Bandwidth + Clone>) {
    if map.len() > MAX_BANDWIDTH_ITEMS {
        let mut bandwidth_list = Vec::from_iter(map.clone());
        sort_by_bandwidth(&mut bandwidth_list);
        for (key, _) in &bandwidth_list[MAX_BANDWIDTH_ITEMS..] {
            map.remove(key);
        }
    }
}

// This is duplicated from table.rs temporarily
fn sort_by_bandwidth<T>(list: &mut Vec<(T, impl Bandwidth)>) {
    list.sort_by_key(|(_, b)| {
        cmp::max(b.get_total_bytes_downloaded(), b.get_total_bytes_uploaded())
    });
}
