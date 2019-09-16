use ::std::collections::BTreeMap;

use crate::traffic::{Connection};
use crate::store::{CurrentConnections, NetworkUtilization};

pub trait Bandwidth {
    fn get_total_bytes_downloaded(&self) -> u128;
    fn get_total_bytes_uploaded(&self) -> u128;
}

#[derive(Default)]
pub struct NetworkData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub connection_count: u128
}

#[derive(Default)]
pub struct ConnectionData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub processes: Vec<String>
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

pub struct UIState {
   pub processes: BTreeMap<String, NetworkData>,
   pub remote_ips: BTreeMap<String, NetworkData>,
   pub connections: BTreeMap<Connection, ConnectionData>
}

impl UIState {
    pub fn new (current_connections: CurrentConnections, network_utilization: &NetworkUtilization) -> Self {
        let mut processes: BTreeMap<String, NetworkData> = BTreeMap::new();
        let mut remote_ips: BTreeMap<String, NetworkData> = BTreeMap::new();
        let mut connections: BTreeMap<Connection, ConnectionData> = BTreeMap::new();
        for (connection, associated_processes) in &current_connections.connections {
            if let Some(connection_bandwidth_utilization) = network_utilization.connections.get(connection) {
                for process in associated_processes.iter() {
                    let data_for_process = processes.entry(process.clone()).or_default();
                    data_for_process.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                    data_for_process.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                    data_for_process.connection_count += 1;
                }
                let connection_data_entry = connections.entry(connection.clone()).or_default();
                let data_for_remote_ip = remote_ips.entry(connection.remote_ip.to_string()).or_default();
                data_for_remote_ip.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                data_for_remote_ip.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                data_for_remote_ip.connection_count += 1;
                connection_data_entry.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                connection_data_entry.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                connection_data_entry.processes.append(&mut associated_processes.clone());
            }
        }
        UIState {
            processes,
            remote_ips,
            connections
        }
    }
}
