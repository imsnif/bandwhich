use ::std::collections::{BTreeMap, HashMap};
use ::std::net::Ipv4Addr;

use crate::network::{Connection, Utilization};

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

#[derive(Default)]
pub struct UIState {
    pub processes: BTreeMap<String, NetworkData>,
    pub remote_ips: BTreeMap<Ipv4Addr, NetworkData>,
    pub connections: BTreeMap<Connection, ConnectionData>,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

impl UIState {
    pub fn new(
        connections_to_procs: HashMap<Connection, String>,
        network_utilization: Utilization,
    ) -> Self {
        let mut processes: BTreeMap<String, NetworkData> = BTreeMap::new();
        let mut remote_ips: BTreeMap<Ipv4Addr, NetworkData> = BTreeMap::new();
        let mut connections: BTreeMap<Connection, ConnectionData> = BTreeMap::new();
        let mut total_bytes_downloaded: u128 = 0;
        let mut total_bytes_uploaded: u128 = 0;
        for (connection, process_name) in connections_to_procs {
            if let Some(connection_bandwidth_utilization) =
                network_utilization.connections.get(&connection)
            {
                let data_for_remote_ip = remote_ips.entry(connection.remote_socket.ip).or_default();
                let connection_data = connections.entry(connection).or_default();
                let data_for_process = processes.entry(process_name.clone()).or_default();

                data_for_process.total_bytes_downloaded +=
                    &connection_bandwidth_utilization.total_bytes_downloaded;
                data_for_process.total_bytes_uploaded +=
                    &connection_bandwidth_utilization.total_bytes_uploaded;
                data_for_process.connection_count += 1;
                connection_data.total_bytes_downloaded +=
                    &connection_bandwidth_utilization.total_bytes_downloaded;
                connection_data.total_bytes_uploaded +=
                    &connection_bandwidth_utilization.total_bytes_uploaded;
                connection_data.process_name = process_name;
                data_for_remote_ip.total_bytes_downloaded +=
                    connection_bandwidth_utilization.total_bytes_downloaded;
                data_for_remote_ip.total_bytes_uploaded +=
                    connection_bandwidth_utilization.total_bytes_uploaded;
                data_for_remote_ip.connection_count += 1;
                total_bytes_downloaded += connection_bandwidth_utilization.total_bytes_downloaded;
                total_bytes_uploaded += connection_bandwidth_utilization.total_bytes_uploaded;
            }
        }
        UIState {
            processes,
            remote_ips,
            connections,
            total_bytes_downloaded,
            total_bytes_uploaded,
        }
    }
}
