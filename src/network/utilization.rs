use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

use crate::network::{Connection, Direction, Segment};

#[derive(Clone)]
pub struct ConnectionInfo {
    pub interface_name: String,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

#[derive(Clone)]
pub struct Utilization {
    pub connections: HashMap<Connection, ConnectionInfo>,
}

impl Utilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        Utilization { connections }
    }
    pub fn clone_and_reset(&mut self) -> Self {
        let clone = self.clone();
        self.connections.clear();
        clone
    }
    pub fn update(&mut self, seg: Segment) {
        let total_bandwidth = self
            .connections
            .entry(seg.connection)
            .or_insert(ConnectionInfo {
                interface_name: seg.interface_name,
                total_bytes_downloaded: 0,
                total_bytes_uploaded: 0,
            });
        match seg.direction {
            Direction::Download => {
                total_bandwidth.total_bytes_downloaded += seg.data_length;
            }
            Direction::Upload => {
                total_bandwidth.total_bytes_uploaded += seg.data_length;
            }
        }
    }
    pub fn remove_ip(&mut self, ips: &Vec<Ipv4Addr>) {
        // might be possible to refactor this part better
        // i still don't understand the whole borrow/own system very well yet
        let placeholder = self.connections.clone();
        for util in placeholder {
            match util.0.remote_socket.ip {
                std::net::IpAddr::V4(ip) => {
                    if ips.contains(&ip) {
                        self.connections.remove_entry(&util.0);
                    }
                }
                std::net::IpAddr::V6(..) => { /* nothing here yet (maybe implement it for ipV6 too) */
                }
            }
        }
    }
    pub fn remove_ip_port(&mut self, ips: &Vec<SocketAddrV4>) {
        // might be possible to refactor this part better
        // i still don't understand the whole borrow/own system very well yet
        let placeholder = self.connections.clone();
        for util in placeholder {
            match util.0.remote_socket.ip {
                std::net::IpAddr::V4(ip) => {
                    if ips.contains(&SocketAddrV4::new(ip, util.0.remote_socket.port)) {
                        self.connections.remove_entry(&util.0);
                    }
                }
                std::net::IpAddr::V6(..) => { /* nothing here yet (maybe implement it for ipV6 too) */
                }
            }
        }
    }
}
