use crate::traffic::{Connection, Protocol};

use ::std::collections::HashMap;
use ::std::net::IpAddr;
use ::netstat::{SocketInfo, ProtocolSocketInfo};

macro_rules! get_ipv4_address {
    ($a:expr) => {
        match $a {
            IpAddr::V4(addr) => Some(addr),
            IpAddr::V6(_) => None
        }
    }
}

macro_rules! build_ipv4_connection {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        match ($a, $b) {
            (Some(local_ip), Some(remote_ip)) => {
                Some(Connection {
                    local_ip,
                    remote_ip,
                    local_port: $c.local_port,
                    remote_port: $c.remote_port,
                    protocol: $d
                })
            },
            (_, _) => None
        }
    }
}

pub struct CurrentConnections {
    pub connections: HashMap<Connection, Vec<String>>
}

impl CurrentConnections {
    pub fn new(get_process_name: &Fn(i32) -> Option<String>, get_open_sockets: &Fn() -> Vec<SocketInfo>) -> Self {
        let sockets_info = get_open_sockets();
        let mut connections = HashMap::new();
        for si in sockets_info {
            match si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_si) => {
                    let local_addr = get_ipv4_address!(tcp_si.local_addr);
                    let remote_addr = get_ipv4_address!(tcp_si.remote_addr);
                    let connection = build_ipv4_connection!(local_addr, remote_addr, tcp_si, Protocol::Tcp);
                    match connection {
                        Some(conn) => {
                            connections.insert(conn, si.associated_pids.iter().map(|pid| get_process_name(*pid as i32).unwrap()).collect()); // TODO: handle None
                        },
                        None => ()
                    }
                },
                ProtocolSocketInfo::Udp(udp_si) => {
                    let local_addr = get_ipv4_address!(udp_si.local_addr);
                    let remote_addr = get_ipv4_address!(udp_si.remote_addr);
                    let connection = build_ipv4_connection!(local_addr, remote_addr, udp_si, Protocol::Udp);
                    match connection {
                        Some(conn) => {
                            connections.insert(conn, si.associated_pids.iter().map(|pid| get_process_name(*pid as i32).unwrap()).collect());
                        },
                        None => ()
                    }
                }
            }
        };
        CurrentConnections {connections}
    }
}
