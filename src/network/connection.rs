use std::{
    collections::HashMap,
    fmt,
    net::{IpAddr, SocketAddr},
};

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Debug, Copy)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    #[allow(dead_code)]
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
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

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Copy)]
pub struct Socket {
    pub ip: IpAddr,
    pub port: u16,
}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Socket { ip, port } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "[{v6}]:{port}"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct LocalSocket {
    pub ip: IpAddr,
    pub port: u16,
    pub protocol: Protocol,
}

impl fmt::Debug for LocalSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LocalSocket { ip, port, protocol } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{protocol}://{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "{protocol}://[{v6}]:{port}"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct Connection {
    pub remote_socket: Socket,
    pub local_socket: LocalSocket,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Connection {
            remote_socket,
            local_socket,
        } = self;
        write!(f, "{local_socket:?} => {remote_socket:?}")
    }
}

pub fn display_ip_or_host(ip: IpAddr, ip_to_host: &HashMap<IpAddr, String>) -> String {
    match ip_to_host.get(&ip) {
        Some(host) => host.clone(),
        None => ip.to_string(),
    }
}

pub fn display_connection_string(
    connection: &Connection,
    ip_to_host: &HashMap<IpAddr, String>,
    interface_name: &str,
) -> String {
    format!(
        "<{interface_name}>:{} => {}:{} ({})",
        connection.local_socket.port,
        display_ip_or_host(connection.remote_socket.ip, ip_to_host),
        connection.remote_socket.port,
        connection.local_socket.protocol,
    )
}

impl Connection {
    pub fn new(
        remote_socket: SocketAddr,
        local_ip: IpAddr,
        local_port: u16,
        protocol: Protocol,
    ) -> Self {
        Connection {
            remote_socket: Socket {
                ip: remote_socket.ip(),
                port: remote_socket.port(),
            },
            local_socket: LocalSocket {
                ip: local_ip,
                port: local_port,
                protocol,
            },
        }
    }
}
