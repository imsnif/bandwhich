use crate::traffic::{Connection};
use crate::store::{CurrentConnections, NetworkUtilization};

use ::std::sync::{Arc, Mutex};
use ::std::collections::HashMap;
// use ::num_traits::{Zero, One};

pub struct NetworkData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub connection_count: u128
}

impl NetworkData {
    fn new () -> Self {
        NetworkData {
            total_bytes_downloaded: 0,
            total_bytes_uploaded: 0,
            connection_count: 0
        }
    }
}

pub struct ConnectionData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub processes: Vec<String>
}

impl ConnectionData {
    fn new () -> Self {
        ConnectionData {
            total_bytes_downloaded: 0,
            total_bytes_uploaded: 0,
            processes: vec![]
        }
    }
}

pub struct UIState {
   pub process_data: HashMap<String, NetworkData>,
   pub remote_ip_data: HashMap<String, NetworkData>,
   pub connection_data: HashMap<Connection, ConnectionData>,
   pub process_names: Vec<String>,
   pub connections: Vec<Connection>,
   pub remote_ips: Vec<String>
}

impl UIState {
    pub fn new (current_connections: CurrentConnections, network_utilization: &Arc<Mutex<NetworkUtilization>>) -> Self {
        let mut process_data: HashMap<String, NetworkData> = HashMap::new();
        let mut remote_ip_data: HashMap<String, NetworkData> = HashMap::new();
        let mut connection_data: HashMap<Connection, ConnectionData> = HashMap::new();
        for (connection, associated_processes) in &current_connections.connections {
            match network_utilization.lock().unwrap().connections.get(connection) {
                Some(connection_bandwidth_utilization) => {
                    for process in associated_processes.iter() {
                        let data_for_process = process_data 
                            .entry(process.clone())
                            .or_insert(NetworkData::new());
                        data_for_process.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                        data_for_process.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                        data_for_process.connection_count += 1;
                    }
                    let connection_data_entry = connection_data
                        .entry(connection.clone())
                        .or_insert(ConnectionData::new());
                    let data_for_remote_ip = remote_ip_data
                        .entry(connection.remote_ip.to_string())
                        .or_insert(NetworkData::new());
                    data_for_remote_ip.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                    data_for_remote_ip.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                    data_for_remote_ip.connection_count += 1;
                    connection_data_entry.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                    connection_data_entry.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                    connection_data_entry.processes.append(&mut associated_processes.clone());

                },
                None => ()
            }
        }
        let mut process_names: Vec<String> = Vec::new();
        let mut connections: Vec<Connection> = Vec::new();
        let mut remote_ips: Vec<String> = Vec::new();
        for (process_name, _) in &process_data {
            process_names.push(process_name.to_string());
        }
        for (connection, _) in &connection_data {
            connections.push(connection.clone());
        }
        for (remote_ip, _) in &remote_ip_data {
            remote_ips.push(remote_ip.to_string());
        }
        process_names.sort();
        connections.sort_by(|a, b| a.partial_cmp(b).unwrap());
        remote_ips.sort();
        UIState {
            process_data,
            remote_ip_data,
            connection_data,
            process_names,
            connections,
            remote_ips
        }
    }
}
