use crate::traffic::{Connection, Protocol};

use ::std::collections::HashMap;
use ::std::net::{IpAddr, Ipv4Addr};
use ::netstat::{SocketInfo, ProtocolSocketInfo};

fn get_ipv4_address(ip: IpAddr) -> Option<Ipv4Addr> {
    match ip {
        IpAddr::V4(addr) => Some(addr),
        IpAddr::V6(_) => None
    }
}

fn build_ipv4_connection (
    local_ip: Option<Ipv4Addr>,
    remote_ip: Option<Ipv4Addr>,
    local_port: u16,
    remote_port: u16,
    protocol: Protocol
) -> Option<Connection> {
    match (local_ip, remote_ip) {
        (Some(local_ip), Some(remote_ip)) => {
            Some(Connection {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol
            })
        },
        (_, _) => None
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
                    let local_addr = get_ipv4_address(tcp_si.local_addr);
                    let remote_addr = get_ipv4_address(tcp_si.remote_addr);
                    if let Some(conn) = build_ipv4_connection(local_addr, remote_addr, tcp_si.local_port, tcp_si.remote_port, Protocol::Tcp) {
                        connections.insert(conn, si.associated_pids.iter().map(|pid| get_process_name(*pid as i32).unwrap()).collect()); // TODO: handle None
                    }
                },
                ProtocolSocketInfo::Udp(udp_si) => {
                    let local_addr = get_ipv4_address(udp_si.local_addr);
                    let remote_addr = get_ipv4_address(udp_si.remote_addr);
                    if let Some(conn) = build_ipv4_connection(local_addr, remote_addr, udp_si.local_port, udp_si.remote_port, Protocol::Udp) {
                        connections.insert(conn, si.associated_pids.iter().map(|pid| get_process_name(*pid as i32).unwrap()).collect());
                    }
                }
            }
        };
        CurrentConnections {connections}
    }
}
