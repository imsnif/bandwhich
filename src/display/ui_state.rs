#![ allow( dead_code, unused_imports ) ]

use ::human_size::{Size};
use ::std::{thread, time};
use ::std::sync::{Arc, Mutex};
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::collections::HashMap;

use crate::traffic::{Connection};
use crate::store::NetworkUtilization;
use crate::current_connections::{CurrentConnections}; // TODO: better naming

use ::num_bigint::{BigUint, ToBigUint, ParseBigIntError};
use ::num_traits::{Zero, One};
use ::num_traits::cast::{ToPrimitive};

use ::netstat::*;

use ::std::io;
use ::std::io::{Write, stdout, stdin};
use ::tui::Terminal;
use ::tui::terminal::Frame;
use ::tui::backend::{Backend, TermionBackend};
use ::tui::widgets::{Widget, Block, Borders, Table, Row};
use ::tui::layout::{Layout, Constraint, Direction, Rect};
use ::tui::style::{Style, Color};
use ::termion::raw::IntoRawMode;
use ::termion::event::{Key, Event, MouseEvent};
use ::termion::input::{TermRead};

pub struct NetworkData {
    pub total_bytes_downloaded: BigUint,
    pub total_bytes_uploaded: BigUint,
    pub connection_count: BigUint
}

impl NetworkData {
    fn new () -> Self {
        NetworkData {
            total_bytes_downloaded: Zero::zero(),
            total_bytes_uploaded: Zero::zero(),
            connection_count: Zero::zero()
        }
    }
}

pub struct ConnectionData {
    pub total_bytes_downloaded: BigUint,
    pub total_bytes_uploaded: BigUint,
    pub processes: Vec<String>
}

impl ConnectionData {
    fn new () -> Self {
        ConnectionData {
            total_bytes_downloaded: Zero::zero(),
            total_bytes_uploaded: Zero::zero(),
            processes: vec![]
        }
    }
}

pub trait IsProcess
{
    fn get_name (&self) -> String;
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
    pub fn new <C, G, T, Z> (create_process: &C, get_sockets_info: &G, mirror_utilization: &Arc<Mutex<NetworkUtilization>>) -> Self where
    C: Fn(i32) -> Result<T, Box<std::error::Error>>,
    G: Fn(AddressFamilyFlags, ProtocolFlags) -> Result<Vec<SocketInfo>, Z>,
    T: IsProcess + std::fmt::Debug,
    Z: std::fmt::Debug
    {
        let current_connections = CurrentConnections::new(create_process, get_sockets_info);
        let mut process_data: HashMap<String, NetworkData> = HashMap::new();
        let mut remote_ip_data: HashMap<String, NetworkData> = HashMap::new();
        let mut connection_data: HashMap<Connection, ConnectionData> = HashMap::new();
        for (connection, associated_processes) in &current_connections.connections {
            match mirror_utilization.lock().unwrap().connections.get(connection) {
                Some(connection_bandwidth_utilization) => {
                    for process in associated_processes.iter() {
                        let data_for_process = process_data 
                            .entry(process.get_name())
                            .or_insert(NetworkData::new());
                        data_for_process.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                        data_for_process.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                        data_for_process.connection_count += &One::one();
                    }
                    let mut process_names: Vec<String> = associated_processes.into_iter().map(|p| p.get_name()).collect();
                    let connection_data_entry = connection_data
                        .entry(connection.clone())
                        .or_insert(ConnectionData::new());
                    let data_for_remote_ip = remote_ip_data
                        .entry(connection.remote_ip.to_string())
                        .or_insert(NetworkData::new());
                    data_for_remote_ip.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                    data_for_remote_ip.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                    data_for_remote_ip.connection_count += &One::one();
                    connection_data_entry.total_bytes_downloaded += &connection_bandwidth_utilization.total_bytes_downloaded;
                    connection_data_entry.total_bytes_uploaded += &connection_bandwidth_utilization.total_bytes_uploaded;
                    connection_data_entry.processes.append(&mut process_names);

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
