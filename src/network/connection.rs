use ::std::collections::HashMap;
use ::std::fmt;
use ::std::net::Ipv4Addr;

use ::std::net::SocketAddr;
use crate::network::Protocol::Tcp;

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Debug)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn from_string(string: &String) -> Option<Self> {
        match string.as_str() {
            "TCP" => Some(Protocol::Tcp),
            "UDP" => Some(Protocol::Udp),
            _ => None,
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Socket {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct Connection {
    pub remote_socket: Socket,
    pub protocol: Protocol,
    pub local_port: u16,
}

pub fn display_ip_or_host(ip: Ipv4Addr, ip_to_host: &HashMap<Ipv4Addr, String>) -> String {
    match ip_to_host.get(&ip) {
        Some(host) => host.clone(),
        None => ip.to_string(),
    }
}

pub fn display_connection_string(
    connection: &Connection,
    ip_to_host: &HashMap<Ipv4Addr, String>,
) -> String {
    format!(
        ":{} => {}:{} ({})",
        connection.local_port,
        display_ip_or_host(connection.remote_socket.ip, ip_to_host),
        connection.remote_socket.port,
        connection.protocol,
    )
}

impl Connection {
    pub fn new(remote_socket: SocketAddr, local_port: u16, protocol: Protocol) -> Option<Self> {
        match remote_socket {
            SocketAddr::V4(remote_socket) => Some(Connection {
                remote_socket: Socket {
                    ip: *remote_socket.ip(),
                    port: remote_socket.port(),
                },
                protocol,
                local_port,
            }),
            _ => None,
        }
    }
}
